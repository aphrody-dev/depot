use std::path::PathBuf;
use depot_core::{CloneOptions, GitSlim, SlimOptions, StyleExt};

pub fn run_slim(repo_path: PathBuf) -> Result<(), String> {
    println!(
        "{} Applying Git Scalar optimizations to {}...",
        "->".cyan().bold(),
        repo_path.display().to_string().yellow()
    );

    let report = GitSlim::slim_repo(&repo_path, &SlimOptions::default())?;

    println!(
        "{} Repository slimmed successfully!",
        "[OK]".green().bold()
    );
    println!(
        "  - Git size: {} -> {}",
        format_size(report.git_dir_size_before_bytes),
        format_size(report.git_dir_size_after_bytes).green()
    );
    println!(
        "  - Scalar registered: {}",
        report.scalar_registered.to_string().cyan()
    );
    println!("  - Optimizations applied:");
    for opt in report.optimizations_applied {
        println!("      * {}", opt);
    }

    Ok(())
}

pub fn run_clone(
    url: String,
    dest: PathBuf,
    branch: Option<String>,
    depth: Option<u32>,
    sparse_paths: Vec<String>,
) -> Result<(), String> {
    println!(
        "{} Cloning {} into {} (minimal weight blobless mode)...",
        "->".cyan().bold(),
        url.yellow(),
        dest.display().to_string().cyan()
    );

    let opts = CloneOptions {
        url,
        dest,
        branch,
        depth,
        blobless: true,
        treeless: false,
        no_tags: true,
        single_branch: true,
        sparse_paths,
    };

    let report = GitSlim::clone_minimal(&opts)?;

    println!(
        "{} Minimal clone complete! Repository size: {}",
        "[OK]".green().bold(),
        format_size(report.git_dir_size_after_bytes).green()
    );

    Ok(())
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
