use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::scanner::IngestedFile;
use crate::tokenizer::Tokenizer;
use crate::tree::build_tree_string;

pub const SEPARATOR: &str = "================================================";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DigestFormat {
    Markdown,
    Xml,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestStats {
    pub repo_name: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub total_bytes: u64,
    pub estimated_tokens: usize,
    pub budget_clamped: bool,
}

pub struct Formatter;

impl Formatter {
    pub fn format_digest(
        root_dir: &Path,
        files: &mut [IngestedFile],
        format: DigestFormat,
        max_tokens: Option<usize>,
    ) -> (String, IngestStats) {
        let repo_name = root_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "codebase".to_string());

        // Sort files deterministically: entry points & configs first, then alphabetical
        files.sort_by(|a, b| {
            let score_a = file_priority(&a.rel_path);
            let score_b = file_priority(&b.rel_path);
            score_b.cmp(&score_a).then_with(|| a.rel_path.cmp(&b.rel_path))
        });

        let mut tree_inputs = Vec::new();
        let mut total_lines = 0;
        let mut total_bytes = 0;

        for f in files.iter() {
            tree_inputs.push((f.rel_path.clone(), f.lines, f.tokens, f.bytes));
            total_lines += f.lines;
            total_bytes += f.bytes;
        }

        let tree_str = build_tree_string(&repo_name, &tree_inputs);
        let tree_tokens = Tokenizer::estimate_tokens(&tree_str);

        // Budget clamping if max_tokens is provided
        let mut budget_clamped = false;
        let mut running_tokens = tree_tokens + 200; // tree + summary overhead

        let mut output = String::new();

        match format {
            DigestFormat::Markdown => {
                output.push_str(&format!("# Codebase Digest: {}\n\n", repo_name));
                output.push_str(&format!("- Files analyzed: {}\n", files.len()));
                output.push_str(&format!("- Total lines: {}\n", total_lines));
                output.push_str(&format!("- Total size: {}\n\n", format_size(total_bytes)));

                output.push_str("## Directory Structure\n\n```text\n");
                output.push_str(&tree_str);
                output.push_str("```\n\n");

                output.push_str("## Source Files\n\n");

                for f in files.iter_mut() {
                    let file_header = format!(
                        "{}\nFILE: {}\n{}\n",
                        SEPARATOR,
                        f.rel_path.to_string_lossy().replace('\\', "/"),
                        SEPARATOR
                    );
                    let header_tokens = Tokenizer::estimate_tokens(&file_header);

                    if let Some(limit) = max_tokens {
                        if running_tokens + header_tokens >= limit {
                            budget_clamped = true;
                            output.push_str(&format!(
                                "{}\n[TRUNCATED: Token limit of {} reached for remaining files]\n\n",
                                SEPARATOR, limit
                            ));
                            break;
                        }

                        if running_tokens + header_tokens + f.tokens > limit {
                            budget_clamped = true;
                            let remaining = limit.saturating_sub(running_tokens + header_tokens);
                            let allowed_chars = remaining * 3;
                            let truncated_content = if f.content.len() > allowed_chars {
                                &f.content[..allowed_chars]
                            } else {
                                &f.content
                            };

                            output.push_str(&file_header);
                            output.push_str(truncated_content);
                            output.push_str("\n\n[... truncated by token budget ...]\n\n");
                            break;
                        }
                    }

                    output.push_str(&file_header);
                    output.push_str(&f.content);
                    output.push_str("\n\n");

                    running_tokens += header_tokens + f.tokens;
                }
            }
            DigestFormat::Xml => {
                output.push_str(&format!("<codebase name=\"{}\">\n", repo_name));
                output.push_str("  <structure>\n");
                output.push_str(&tree_str);
                output.push_str("  </structure>\n");
                output.push_str("  <files>\n");

                for f in files.iter() {
                    output.push_str(&format!(
                        "    <file path=\"{}\" lines=\"{}\" tokens=\"{}\">\n<![CDATA[\n",
                        f.rel_path.to_string_lossy().replace('\\', "/"),
                        f.lines,
                        f.tokens
                    ));
                    output.push_str(&f.content);
                    output.push_str("\n]]>\n    </file>\n");
                }
                output.push_str("  </files>\n</codebase>\n");
            }
            DigestFormat::Json => {
                let serialized = serde_json::to_string_pretty(&files).unwrap_or_default();
                output.push_str(&serialized);
            }
        }

        let total_tokens = Tokenizer::estimate_tokens(&output);

        let stats = IngestStats {
            repo_name,
            total_files: files.len(),
            total_lines,
            total_bytes,
            estimated_tokens: total_tokens,
            budget_clamped,
        };

        (output, stats)
    }
}

fn file_priority(path: &Path) -> u32 {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match name {
        "README.md" | "ARCHITECTURE.md" => 100,
        "Cargo.toml" | "package.json" | "pyproject.toml" | "justfile" => 90,
        "main.rs" | "lib.rs" | "index.ts" | "main.py" => 80,
        _ => 50,
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
