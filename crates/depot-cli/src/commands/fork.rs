use std::path::PathBuf;
use depot_core::{ForkManager, SyncStrategy, StyleExt};

pub fn run_fork_status(repo_path: PathBuf) -> Result<(), String> {
    let st = ForkManager::get_status(&repo_path)?;

    println!(
        "{} Fork Status for {}:",
        "[FORK]".cyan().bold(),
        repo_path.display()
    );
    println!("  Branch:   {}", st.current_branch.green().bold());
    println!(
        "  Origin:   {}",
        st.origin_url.as_deref().unwrap_or("None").yellow()
    );
    println!(
        "  Upstream: {}",
        st.upstream_url.as_deref().unwrap_or("None").yellow()
    );
    println!(
        "  Sync:     {} ahead, {} behind",
        st.ahead_count.to_string().cyan(),
        st.behind_count.to_string().magenta()
    );

    if st.has_diverged {
        println!(
            "  {} Branches have diverged! Consider running `depot fork sync --strategy rebase`.",
            "[WARN]".red().bold()
        );
    }

    Ok(())
}

pub fn run_fork_set_upstream(repo_path: PathBuf, url: String) -> Result<(), String> {
    ForkManager::set_upstream(&repo_path, &url)?;
    println!(
        "{} Upstream remote set to: {}",
        "[OK]".green().bold(),
        url.yellow()
    );
    Ok(())
}

pub fn run_fork_sync(
    repo_path: PathBuf,
    branch: Option<String>,
    strategy_str: String,
) -> Result<(), String> {
    let strat = match strategy_str.to_lowercase().as_str() {
        "rebase" => SyncStrategy::Rebase,
        "merge" => SyncStrategy::Merge,
        _ => SyncStrategy::FastForwardOnly,
    };

    println!(
        "{} Syncing branch {:?} with upstream...",
        "->".cyan().bold(),
        branch
    );

    let res = ForkManager::sync_upstream(&repo_path, branch.as_deref(), strat)?;
    println!("{}\n{}", "[OK]".green().bold(), res);
    Ok(())
}
