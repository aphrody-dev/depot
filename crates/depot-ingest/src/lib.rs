pub mod formatter;
pub mod scanner;
pub mod tokenizer;
pub mod tree;

pub use formatter::{DigestFormat, Formatter, IngestStats, SEPARATOR};
pub use scanner::{IngestedFile, Scanner};
pub use tokenizer::Tokenizer;
pub use tree::build_tree_string;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub respect_gitignore: bool,
    pub respect_llmignore: bool,
    pub max_file_size_bytes: u64,
    pub max_tokens: Option<usize>,
    pub format: DigestFormat,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
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

    Formatter::format_digest(root_dir, &mut files, config.format, config.max_tokens)
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
    fn test_formatter_markdown() {
        let mut files = vec![IngestedFile {
            rel_path: PathBuf::from("src/lib.rs"),
            content: "pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
            lines: 1,
            bytes: 43,
            tokens: 15,
            is_truncated: false,
        }];

        let (digest, stats) = Formatter::format_digest(
            Path::new("dummy"),
            &mut files,
            DigestFormat::Markdown,
            None,
        );

        assert_eq!(stats.total_files, 1);
        assert!(digest.contains(SEPARATOR));
        assert!(digest.contains("FILE: src/lib.rs"));
    }
}
