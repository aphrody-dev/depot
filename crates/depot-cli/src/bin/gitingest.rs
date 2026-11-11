use std::path::PathBuf;
use argh::FromArgs;
use depot_core::StyleExt;
use depot_ingest::{ingest_codebase, DigestFormat, IngestConfig};

#[derive(FromArgs)]
/// Ultra-fast native Rust codebase-to-LLM prompt digest generator (100x faster than python gitingest)
struct IngestCli {
    /// target directory to ingest (defaults to ".")
    #[argh(positional)]
    path: Option<PathBuf>,

    /// output format: markdown, xml, json (default: markdown)
    #[argh(option, short = 'f', default = "String::from(\"markdown\")")]
    format: String,

    /// max token limit for output
    #[argh(option, short = 'm')]
    max_tokens: Option<usize>,

    /// save output to file instead of stdout
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,

    /// include glob patterns
    #[argh(option, short = 'i')]
    include: Vec<String>,

    /// exclude glob patterns
    #[argh(option, short = 'e')]
    exclude: Vec<String>,

    /// produce high-density symbol outline only
    #[argh(switch)]
    outline: bool,
}

fn main() {
    depot_core::enable_vt_processing();
    let cli: IngestCli = argh::from_env();
    let target_path = cli.path.unwrap_or_else(|| PathBuf::from("."));

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
        outline_only: cli.outline,
    };

    let (digest, stats) = ingest_codebase(&target_path, &config);

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
