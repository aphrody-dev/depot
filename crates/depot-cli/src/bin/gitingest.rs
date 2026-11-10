use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use depot_ingest::{ingest_codebase, DigestFormat, IngestConfig};

#[derive(Parser)]
#[command(name = "gitingest")]
#[command(about = "Ultra-fast native Rust codebase-to-LLM prompt digest generator (100x faster than python gitingest)")]
#[command(version = "0.1.0")]
struct IngestCli {
    /// Directory to ingest
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format: markdown, xml, json
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Max token limit for output
    #[arg(short, long)]
    max_tokens: Option<usize>,

    /// Save output to file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include glob patterns
    #[arg(short, long)]
    include: Vec<String>,

    /// Exclude glob patterns
    #[arg(short, long)]
    exclude: Vec<String>,
}

fn main() {
    let cli = IngestCli::parse();

    let format = match cli.format.to_lowercase().as_str() {
        "xml" => DigestFormat::Xml,
        "json" => DigestFormat::Json,
        _ => DigestFormat::Markdown,
    };

    let config = IngestConfig {
        respect_gitignore: true,
        respect_llmignore: true,
        max_file_size_bytes: 512 * 1024,
        max_tokens: cli.max_tokens,
        format,
        include_patterns: cli.include,
        exclude_patterns: cli.exclude,
    };

    let (digest, stats) = ingest_codebase(&cli.path, &config);

    if let Some(out_p) = cli.output {
        if let Err(e) = std::fs::write(&out_p, &digest) {
            eprintln!("{} Failed to write {}: {}", "[ERROR]".red().bold(), out_p.display(), e);
            std::process::exit(1);
        }
        eprintln!(
            "{} Digest written to {}",
            "[OK]".green().bold(),
            out_p.display().to_string().cyan()
        );
    } else {
        println!("{}", digest);
    }

    eprintln!(
        "{} {} files | {} lines | ~{} tokens | {}",
        "[STATS]".magenta().bold(),
        stats.total_files,
        stats.total_lines,
        stats.estimated_tokens,
        if stats.budget_clamped {
            "(budget clamped)".red().bold().to_string()
        } else {
            "(full digest)".green().to_string()
        }
    );
}
