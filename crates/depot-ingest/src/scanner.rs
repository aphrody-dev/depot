use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use depot_core::SmartFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestedFile {
    pub rel_path: PathBuf,
    pub content: String,
    pub lines: usize,
    pub bytes: u64,
    pub tokens: usize,
    pub is_truncated: bool,
}

pub struct Scanner;

impl Scanner {
    pub fn is_binary_file(path: &Path) -> bool {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(_) => return true,
        };
        let mut buffer = [0u8; 1024];
        let n = f.read(&mut buffer).unwrap_or(0);
        buffer[..n].contains(&0)
    }

    pub fn scan(
        root_dir: &Path,
        respect_gitignore: bool,
        respect_llmignore: bool,
        max_file_size_bytes: u64,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Vec<IngestedFile> {
        let walker = SmartFilter::build_walker(
            root_dir,
            respect_gitignore,
            respect_llmignore,
            exclude_patterns,
        );

        let candidate_paths: Vec<PathBuf> = walker
            .build()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|e| e.into_path())
            .collect();

        candidate_paths
            .into_par_iter()
            .filter_map(|full_path| {
                let rel = match full_path.strip_prefix(root_dir) {
                    Ok(r) => r.to_path_buf(),
                    Err(_) => full_path.clone(),
                };

                // Check includes if specified
                if !include_patterns.is_empty() {
                    let rel_str = rel.to_string_lossy();
                    let matched = include_patterns.iter().any(|pat| {
                        globset::Glob::new(pat)
                            .map(|g| g.compile_matcher().is_match(&*rel_str))
                            .unwrap_or(false)
                    });
                    if !matched {
                        return None;
                    }
                }

                // Check file size
                let metadata = full_path.metadata().ok()?;
                if metadata.len() > max_file_size_bytes {
                    return None;
                }

                // Check binary
                if Self::is_binary_file(&full_path) {
                    return None;
                }

                // Read content
                let content = std::fs::read_to_string(&full_path).ok()?;
                let lines = content.lines().count();
                let bytes = metadata.len();
                let tokens = crate::tokenizer::Tokenizer::estimate_tokens(&content);

                Some(IngestedFile {
                    rel_path: rel,
                    content,
                    lines,
                    bytes,
                    tokens,
                    is_truncated: false,
                })
            })
            .collect()
    }
}
