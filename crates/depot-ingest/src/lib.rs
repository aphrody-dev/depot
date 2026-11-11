pub mod formatter;
pub mod scanner;
pub mod tokenizer;
pub mod tree;

pub use formatter::{DigestFormat, Formatter, IngestStats, SEPARATOR};
pub use scanner::{IngestedFile, Scanner};
pub use tokenizer::Tokenizer;
pub use tree::build_tree_string;

use std::path::Path;
use depot_core::{CloneOptions, GitSlim};

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub respect_gitignore: bool,
    pub respect_llmignore: bool,
    pub max_file_size_bytes: u64,
    pub max_tokens: Option<usize>,
    pub format: DigestFormat,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub outline_only: bool,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            respect_gitignore: true,
            respect_llmignore: true,
            max_file_size_bytes: 512 * 1024,
            max_tokens: None,
            format: DigestFormat::Markdown,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            outline_only: false,
        }
    }
}

pub fn ingest_codebase(root_dir: &Path, config: &IngestConfig) -> (String, IngestStats) {
    let mut files = Scanner::scan(
        root_dir,
        config.respect_gitignore,
        config.respect_llmignore,
        config.max_file_size_bytes,
        &config.include_patterns,
        &config.exclude_patterns,
    );

    Formatter::format_digest(
        root_dir,
        &mut files,
        config.format,
        config.max_tokens,
        config.outline_only,
    )
}

pub fn ingest_remote_url(url: &str, config: &IngestConfig) -> Result<(String, IngestStats), String> {
    let clean_url = url.trim().trim_end_matches('/');
    if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
        return Err(format!("Invalid remote repository URL: {}", url));
    }

    let repo_name = clean_url
        .rsplit('/')
        .next()
        .unwrap_or("remote_repo")
        .trim_end_matches(".git");

    let tmp_dest = std::env::temp_dir().join(format!("depot_remote_{}_{}", repo_name, std::process::id()));

    let clone_opts = CloneOptions {
        url: clean_url.to_string(),
        dest: tmp_dest.clone(),
        branch: None,
        depth: Some(1),
        blobless: false,
        treeless: false,
        no_tags: true,
        single_branch: true,
        sparse_paths: Vec::new(),
    };

    GitSlim::clone_minimal(&clone_opts)
        .map_err(|e| format!("Failed to clone remote repository {}: {}", clean_url, e))?;

    let res = ingest_codebase(&tmp_dest, config);
    let _ = std::fs::remove_dir_all(&tmp_dest);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_tokenizer_estimation() {
        let code = "fn main() {\n    println!(\"Hello world\");\n}\n";
        let tokens = Tokenizer::estimate_tokens(code);
        assert!(tokens >= 5 && tokens <= 20);
    }

    #[test]
    fn test_tree_builder() {
        let files = vec![
            (PathBuf::from("src/main.rs"), 10, 40, 150),
            (PathBuf::from("Cargo.toml"), 5, 20, 80),
        ];
        let tree = build_tree_string("my-crate", &files);
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
        assert!(tree.contains("Cargo.toml"));
    }

    #[test]
    fn test_formatter_markdown_and_outline() {
        let mut files = vec![IngestedFile {
            rel_path: PathBuf::from("src/lib.rs"),
            content: "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub struct Calc;\n".to_string(),
            lines: 2,
            bytes: 60,
            tokens: 20,
            is_truncated: false,
        }];

        let (digest, stats) = Formatter::format_digest(
            Path::new("dummy"),
            &mut files,
            DigestFormat::Markdown,
            None,
            false,
        );

        assert_eq!(stats.total_files, 1);
        assert!(digest.contains(SEPARATOR));
        assert!(digest.contains("FILE: src/lib.rs"));

        // Outline test
        let (outline_digest, _) = Formatter::format_digest(
            Path::new("dummy"),
            &mut files,
            DigestFormat::Markdown,
            None,
            true,
        );
        assert!(outline_digest.contains("[OUTLINE"));
        assert!(outline_digest.contains("add"));
    }
}
