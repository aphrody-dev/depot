use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkStatus {
    pub current_branch: String,
    pub origin_url: Option<String>,
    pub upstream_url: Option<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub has_diverged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    FastForwardOnly,
    Rebase,
    Merge,
}

pub struct ForkManager;

impl ForkManager {
    pub fn get_status(repo_path: &Path) -> Result<ForkStatus, String> {
        // Current branch
        let branch_out = Command::new("git")
            .current_dir(repo_path)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| format!("Failed to get branch: {}", e))?;
        let current_branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();

        // Remotes
        let remotes_out = Command::new("git")
            .current_dir(repo_path)
            .args(["remote", "-v"])
            .output()
            .map_err(|e| format!("Failed to get remotes: {}", e))?;
        let remotes_text = String::from_utf8_lossy(&remotes_out.stdout);

        let mut origin_url = None;
        let mut upstream_url = None;

        for line in remotes_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if parts[0] == "origin" && origin_url.is_none() {
                    origin_url = Some(parts[1].to_string());
                } else if parts[0] == "upstream" && upstream_url.is_none() {
                    upstream_url = Some(parts[1].to_string());
                }
            }
        }

        // Ahead / Behind counts against upstream if configured, else origin
        let target_remote = if upstream_url.is_some() { "upstream" } else { "origin" };
        let rev_spec = format!("{}/{}...HEAD", target_remote, current_branch);

        let rev_list = Command::new("git")
            .current_dir(repo_path)
            .args(["rev-list", "--left-right", "--count", &rev_spec])
            .output();

        let mut ahead_count = 0;
        let mut behind_count = 0;

        if let Ok(out) = rev_list {
            if out.status.success() {
                let counts = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = counts.split_whitespace().collect();
                if parts.len() >= 2 {
                    behind_count = parts[0].parse().unwrap_or(0);
                    ahead_count = parts[1].parse().unwrap_or(0);
                }
            }
        }

        Ok(ForkStatus {
            current_branch,
            origin_url,
            upstream_url,
            ahead_count,
            behind_count,
            has_diverged: ahead_count > 0 && behind_count > 0,
        })
    }

    pub fn set_upstream(repo_path: &Path, url: &str) -> Result<(), String> {
        // Check if upstream remote exists
        let check = Command::new("git")
            .current_dir(repo_path)
            .args(["remote", "get-url", "upstream"])
            .status();

        let exists = check.map(|s| s.success()).unwrap_or(false);

        let status = if exists {
            Command::new("git")
                .current_dir(repo_path)
                .args(["remote", "set-url", "upstream", url])
                .status()
        } else {
            Command::new("git")
                .current_dir(repo_path)
                .args(["remote", "add", "upstream", url])
                .status()
        };

        match status {
            Ok(st) if st.success() => Ok(()),
            _ => Err("Failed to set upstream remote".to_string()),
        }
    }

    pub fn sync_upstream(
        repo_path: &Path,
        branch: Option<&str>,
        strategy: SyncStrategy,
    ) -> Result<String, String> {
        let branch_name = match branch {
            Some(b) => b.to_string(),
            None => {
                let out = Command::new("git")
                    .current_dir(repo_path)
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .output()
                    .map_err(|e| format!("Branch query failed: {}", e))?;
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
        };

        // Fetch upstream
        let fetch_st = Command::new("git")
            .current_dir(repo_path)
            .args(["fetch", "upstream", &branch_name])
            .status()
            .map_err(|e| format!("Fetch failed: {}", e))?;

        if !fetch_st.success() {
            return Err(format!("Failed to fetch upstream/{}", branch_name));
        }

        let upstream_ref = format!("upstream/{}", branch_name);

        match strategy {
            SyncStrategy::FastForwardOnly => {
                let merge_out = Command::new("git")
                    .current_dir(repo_path)
                    .args(["merge", "--ff-only", &upstream_ref])
                    .output()
                    .map_err(|e| format!("Merge failed: {}", e))?;
                if !merge_out.status.success() {
                    return Err(String::from_utf8_lossy(&merge_out.stderr).to_string());
                }
                Ok(String::from_utf8_lossy(&merge_out.stdout).to_string())
            }
            SyncStrategy::Rebase => {
                let rebase_out = Command::new("git")
                    .current_dir(repo_path)
                    .args(["rebase", &upstream_ref])
                    .output()
                    .map_err(|e| format!("Rebase failed: {}", e))?;
                if !rebase_out.status.success() {
                    return Err(String::from_utf8_lossy(&rebase_out.stderr).to_string());
                }
                Ok(String::from_utf8_lossy(&rebase_out.stdout).to_string())
            }
            SyncStrategy::Merge => {
                let merge_out = Command::new("git")
                    .current_dir(repo_path)
                    .args(["merge", &upstream_ref])
                    .output()
                    .map_err(|e| format!("Merge failed: {}", e))?;
                if !merge_out.status.success() {
                    return Err(String::from_utf8_lossy(&merge_out.stderr).to_string());
                }
                Ok(String::from_utf8_lossy(&merge_out.stdout).to_string())
            }
        }
    }
}
