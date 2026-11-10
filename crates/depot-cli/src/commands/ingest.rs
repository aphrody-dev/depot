use std::path::PathBuf;
use depot_core::StyleExt;
use depot_ingest::{ingest_codebase, DigestFormat, IngestConfig};

pub fn run_ingest(
    target_path: PathBuf,
    format_str: String,
    max_tokens: Option<usize>,
    output_file: Option<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
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
    };

    println!(
        "{} Ingesting codebase at {}...",
        "->".cyan().bold(),
        target_path.display().to_string().yellow()
    );

    let (digest, stats) = ingest_codebase(&target_path, &config);

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
