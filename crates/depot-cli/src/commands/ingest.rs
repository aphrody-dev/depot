use std::path::PathBuf;
use depot_core::StyleExt;
use depot_ingest::{ingest_codebase, ingest_remote_url, DigestFormat, IngestConfig};

pub fn run_ingest(
    target_str: String,
    format_str: String,
    max_tokens: Option<usize>,
    output_file: Option<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    outline: bool,
) -> Result<(), String> {
    let format = match format_str.to_lowercase().as_str() {
        "xml" => DigestFormat::Xml,
        "json" => DigestFormat::Json,
        _ => DigestFormat::Markdown,
    };

    let config = IngestConfig {
        respect_gitignore: true,
        respect_llmignore: true,
        max_file_size_bytes: 512 * 1024,
        max_tokens,
        format,
        include_patterns: include,
        exclude_patterns: exclude,
        outline_only: outline,
    };

    let is_remote = target_str.starts_with("http://") || target_str.starts_with("https://");

    println!(
        "{} Ingesting {} at {}...",
        "->".cyan().bold(),
        if is_remote { "remote repo" } else { "codebase" },
        target_str.yellow()
    );

    let (digest, stats) = if is_remote {
        ingest_remote_url(&target_str, &config)?
    } else {
        let path = PathBuf::from(&target_str);
        ingest_codebase(&path, &config)
    };

    if let Some(out_p) = output_file {
        std::fs::write(&out_p, &digest).map_err(|e| format!("Failed to write {}: {}", out_p.display(), e))?;
        println!(
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

    Ok(())
}
