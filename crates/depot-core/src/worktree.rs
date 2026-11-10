use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub head_commit: String,
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_locked: bool,
    pub is_prunable: bool,
}

pub struct WorktreeManager;

impl WorktreeManager {
    pub fn list(repo_path: &Path) -> Result<Vec<WorktreeInfo>, String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["worktree", "list", "--porcelain"])
            .output()
            .map_err(|e| format!("Failed to list worktrees: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree list failed: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();
        let mut current_path: Option<PathBuf> = None;
        let mut current_head = String::new();
        let mut current_branch: Option<String> = None;
        let mut is_bare = false;
        let mut is_locked = false;
        let mut is_prunable = false;

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                if let Some(path) = current_path.take() {
                    worktrees.push(WorktreeInfo {
                        path,
                        head_commit: current_head.clone(),
                        branch: current_branch.take(),
                        is_bare,
                        is_locked,
                        is_prunable,
                    });
                }
                current_head.clear();
                is_bare = false;
                is_locked = false;
                is_prunable = false;
                continue;
            }

            if let Some(stripped) = line.strip_prefix("worktree ") {
                current_path = Some(PathBuf::from(stripped));
            } else if let Some(stripped) = line.strip_prefix("HEAD ") {
                current_head = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(stripped.to_string());
            } else if line == "bare" {
                is_bare = true;
            } else if line.starts_with("locked") {
                is_locked = true;
            } else if line.starts_with("prunable") {
                is_prunable = true;
            }
        }

        if let Some(path) = current_path.take() {
            worktrees.push(WorktreeInfo {
                path,
                head_commit: current_head,
                branch: current_branch,
                is_bare,
                is_locked,
                is_prunable,
            });
        }

        Ok(worktrees)
    }

    pub fn add(
        repo_path: &Path,
        dest_path: &Path,
        branch_name: &str,
        create_new: bool,
    ) -> Result<WorktreeInfo, String> {
        let mut args = vec!["worktree".to_string(), "add".to_string()];

        if create_new {
            args.push("-b".to_string());
            args.push(branch_name.to_string());
        }

        args.push(dest_path.to_string_lossy().to_string());

        if !create_new {
            args.push(branch_name.to_string());
        }

        let output = Command::new("git")
            .current_dir(repo_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run git worktree add: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree add failed: {}", err));
        }

        // Fetch commit hash of newly created worktree
        let head_out = Command::new("git")
            .current_dir(dest_path)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;

        let head = String::from_utf8_lossy(&head_out.stdout).trim().to_string();

        Ok(WorktreeInfo {
            path: dest_path.to_path_buf(),
            head_commit: head,
            branch: Some(branch_name.to_string()),
            is_bare: false,
            is_locked: false,
            is_prunable: false,
        })
    }

    pub fn remove(repo_path: &Path, worktree_path: &Path, force: bool) -> Result<(), String> {
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        let path_str = worktree_path.to_string_lossy();
        args.push(&path_str);

        let output = Command::new("git")
            .current_dir(repo_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to remove worktree: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree remove failed: {}", err));
        }

        Ok(())
    }

    pub fn prune(repo_path: &Path) -> Result<(), String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["worktree", "prune"])
            .output()
            .map_err(|e| format!("Failed to prune worktrees: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree prune failed: {}", err));
        }

        Ok(())
    }
}
