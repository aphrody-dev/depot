use std::path::PathBuf;
use depot_core::{StyleExt, WorktreeManager};

pub fn run_worktree_list(repo_path: PathBuf) -> Result<(), String> {
    let list = WorktreeManager::list(&repo_path)?;

    println!(
        "{} Worktrees for {}:",
        "[WORKTREES]".cyan().bold(),
        repo_path.display()
    );

    for (idx, wt) in list.iter().enumerate() {
        let branch_disp = wt.branch.as_deref().unwrap_or("<detached>");
        let head_short = if wt.head_commit.len() >= 7 {
            &wt.head_commit[..7]
        } else {
            &wt.head_commit
        };

        println!(
            "  [{}] {} ({}) -> {}",
            idx + 1,
            wt.path.display().to_string().yellow(),
            head_short.cyan(),
            branch_disp.green().bold()
        );
    }

    Ok(())
}

pub fn run_worktree_add(
    repo_path: PathBuf,
    dest_path: PathBuf,
    branch_name: String,
    create_new: bool,
) -> Result<(), String> {
    println!(
        "{} Adding worktree at {} on branch {}...",
        "->".cyan().bold(),
        dest_path.display().to_string().yellow(),
        branch_name.cyan().bold()
    );

    let info = WorktreeManager::add(&repo_path, &dest_path, &branch_name, create_new)?;

    println!(
        "{} Worktree successfully created at {} ({})",
        "[OK]".green().bold(),
        info.path.display().to_string().yellow(),
        info.head_commit.chars().take(7).collect::<String>().cyan()
    );

    Ok(())
}

pub fn run_worktree_remove(
    repo_path: PathBuf,
    worktree_path: PathBuf,
    force: bool,
) -> Result<(), String> {
    WorktreeManager::remove(&repo_path, &worktree_path, force)?;
    println!(
        "{} Worktree removed: {}",
        "[OK]".green().bold(),
        worktree_path.display().to_string().yellow()
    );
    Ok(())
}

pub fn run_worktree_prune(repo_path: PathBuf) -> Result<(), String> {
    WorktreeManager::prune(&repo_path)?;
    println!(
        "{} Stale worktrees pruned.",
        "[OK]".green().bold()
    );
    Ok(())
}
