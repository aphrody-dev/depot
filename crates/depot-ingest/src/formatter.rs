use std::path::Path;
use serde::{Deserialize, Serialize};
use depot_core::GixEngine;
use depot_treesitter::TreeSitterEngine;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
}

pub struct Formatter;

impl Formatter {
    pub fn generate_outline(path: &Path, content: &str) -> Option<String> {
        let summary = TreeSitterEngine::parse_file(path, content)?;
        let mut out = String::new();
        out.push_str(&format!(
            "// [OUTLINE - {} symbols, {} lines]\n",
            summary.symbols.len(),
            summary.line_count
        ));
        for imp in &summary.imports {
            out.push_str(&format!("// import {}\n", imp.source_module));
        }
        for sym in &summary.symbols {
            let sig = sym.signature.as_deref().unwrap_or(&sym.name);
            out.push_str(&format!("// {}: {}\n", sym.span.source_location(), sig));
        }
        Some(out)
    }

    pub fn format_digest(
        root_dir: &Path,
        files: &mut [IngestedFile],
        format: DigestFormat,
        max_tokens: Option<usize>,
        outline_only: bool,
    ) -> (String, IngestStats) {
        let repo_name = root_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "codebase".to_string());

        // Inspect git telemetry natively via GixEngine
        let git_telemetry = GixEngine::inspect(root_dir, 5).ok();
        let (git_branch, git_commit) = if let Some(ref tel) = git_telemetry {
            (
                Some(tel.branch.clone()),
                tel.head_commit.as_ref().map(|c| {
                    let subj = tel.head_subject.as_deref().unwrap_or("");
                    format!("{} ({})", &c[..c.len().min(8)], subj)
                }),
            )
        } else {
            (None, None)
        };

        // If outline_only is enabled, replace code file bodies with AST symbol outline
        if outline_only {
            for f in files.iter_mut() {
                if let Some(outline) = Self::generate_outline(&f.rel_path, &f.content) {
                    f.content = outline;
                    f.lines = f.content.lines().count();
                    f.bytes = f.content.len() as u64;
                    f.tokens = Tokenizer::estimate_tokens(&f.content);
                }
            }
        }

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
        let mut running_tokens = tree_tokens + 250; // tree + metadata overhead

        let mut output = String::new();

        match format {
            DigestFormat::Markdown => {
                output.push_str(&format!("# Codebase Digest: {}\n\n", repo_name));
                output.push_str(&format!("- Files analyzed: {}\n", files.len()));
                output.push_str(&format!("- Total lines: {}\n", total_lines));
                output.push_str(&format!("- Total size: {}\n", format_size(total_bytes)));

                if let Some(ref br) = git_branch {
                    output.push_str(&format!("- Git branch: `{}`\n", br));
                }
                if let Some(ref c) = git_commit {
                    output.push_str(&format!("- Git HEAD commit: `{}`\n", c));
                }
                if outline_only {
                    output.push_str("- Digest mode: `AST-Smart Outline`\n");
                }
                output.push('\n');

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
                            // Attempt AST outline for remaining portion
                            let outline = Self::generate_outline(&f.rel_path, &f.content);
                            output.push_str(&file_header);

                            if let Some(outl) = outline {
                                output.push_str("// [AST-Smart Truncation: Full content exceeds budget, outline shown below]\n");
                                output.push_str(&outl);
                            } else {
                                let remaining = limit.saturating_sub(running_tokens + header_tokens);
                                let allowed_chars = remaining * 3;
                                let truncated_content = if f.content.len() > allowed_chars {
                                    &f.content[..allowed_chars]
                                } else {
                                    &f.content
                                };
                                output.push_str(truncated_content);
                                output.push_str("\n\n[... truncated by token budget ...]\n\n");
                            }
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
                if let Some(ref br) = git_branch {
                    output.push_str(&format!("  <git_branch>{}</git_branch>\n", br));
                }
                if let Some(ref c) = git_commit {
                    output.push_str(&format!("  <git_head>{}</git_head>\n", c));
                }
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
            git_branch,
            git_commit,
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
