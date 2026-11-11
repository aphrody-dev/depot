//! Pure-Rust Gitoxide (`gix` v0.61) zero-subprocess repository engine.
//!
//! Opens repositories natively in-process without spawning `git.exe`, providing:
//! - Instant repository discovery (`gix::discover`)
//! - HEAD commit SHA, symbolic branch name, detached state
//! - Remote configuration (`origin`, `upstream`, `gitlab`, `google`, etc.)
//! - Commit log DAG walk (`rev_walk`) with author, timestamp, and summary
//! - Tracked Git tree / blob enumeration directly from the Object Database (ODB)

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GixCommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp_secs: i64,
    pub subject: String,
    pub parent_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GixTreeEntry {
    pub rel_path: String,
    pub oid: String,
    pub is_blob: bool,
    pub is_submodule: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GixRepoTelemetry {
    pub workdir: String,
    pub git_dir: String,
    pub is_bare: bool,
    pub head_ref: Option<String>,
    pub branch: String,
    pub head_commit: Option<String>,
    pub head_subject: Option<String>,
    pub remotes: BTreeMap<String, String>,
    pub recent_commits: Vec<GixCommitSummary>,
    pub tracked_entry_count: usize,
    pub submodule_count: usize,
    pub has_jujutsu: bool,
}

pub struct GixEngine;

impl GixEngine {
    /// Inspect a Git repository natively via `gix` (zero external subprocesses).
    pub fn inspect(path: &Path, commit_limit: usize) -> Result<GixRepoTelemetry, String> {
        let repo = gix::discover(path)
            .map_err(|e| format!("gix::discover failed for {}: {}", path.display(), e))?;

        let git_dir = repo.git_dir().to_path_buf();
        let workdir_path: PathBuf = repo
            .work_dir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| git_dir.clone());
        let is_bare = repo.is_bare();
        let has_jujutsu = workdir_path.join(".jj").is_dir();

        // HEAD & branch
        let mut head = repo.head().map_err(|e| format!("gix head error: {}", e))?;
        let head_ref = head.referent_name().map(|n| n.as_bstr().to_string());
        let branch = head_ref
            .as_deref()
            .and_then(|r| r.strip_prefix("refs/heads/"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if head.is_detached() {
                    "HEAD (detached)".to_string()
                } else {
                    head_ref.clone().unwrap_or_else(|| "unborn".to_string())
                }
            });

        let head_commit_obj = head.peel_to_commit_in_place().ok();
        let head_commit = head_commit_obj.as_ref().map(|c| c.id.to_string());
        let head_subject = head_commit_obj.as_ref().and_then(|c| {
            c.message_raw_sloppy()
                .to_string()
                .lines()
                .next()
                .map(|s| s.trim().to_string())
        });

        // Remotes
        let mut remotes = BTreeMap::new();
        for name_bstr in repo.remote_names() {
            let name_str = name_bstr.to_string();
            if let Ok(rem) = repo.find_remote(name_bstr.as_ref())
                && let Some(url) = rem.url(gix::remote::Direction::Fetch)
            {
                remotes.insert(name_str, url.to_bstring().to_string());
            }
        }

        // Recent commit walk via gix rev_walk
        let mut recent_commits = Vec::new();
        if commit_limit > 0
            && let Some(ref commit) = head_commit_obj
            && let Ok(walk) = repo.rev_walk([commit.id]).all()
        {
            for info_res in walk.take(commit_limit) {
                let Ok(info) = info_res else {
                    break;
                };
                let sha = info.id.to_string();
                let short_sha = sha.chars().take(10).collect::<String>();
                let parent_count = info.parent_ids.len();

                if let Ok(obj) = repo.find_object(info.id)
                    && let Ok(c_obj) = obj.try_into_commit()
                {
                    let (author_name, author_email, ts) = match c_obj.author() {
                        Ok(act) => (
                            act.name.to_string(),
                            act.email.to_string(),
                            act.time.seconds,
                        ),
                        Err(_) => ("unknown".to_string(), String::new(), 0),
                    };
                    let subject = c_obj
                        .message_raw_sloppy()
                        .to_string()
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    recent_commits.push(GixCommitSummary {
                        sha,
                        short_sha,
                        author_name,
                        author_email,
                        timestamp_secs: ts,
                        subject,
                        parent_count,
                    });
                }
            }
        }

        // Count root-tree / recursive tracked entries (fast ODB tree walk, bounded to avoid Chromium 500k lag)
        let (tracked_entry_count, submodule_count) = match head_commit_obj {
            Some(ref c) => Self::count_tree_entries(&repo, c, 25_000),
            None => (0, 0),
        };

        Ok(GixRepoTelemetry {
            workdir: crate::path::normalize_posix(&workdir_path),
            git_dir: crate::path::normalize_posix(&git_dir),
            is_bare,
            head_ref,
            branch,
            head_commit,
            head_subject,
            remotes,
            recent_commits,
            tracked_entry_count,
            submodule_count,
            has_jujutsu,
        })
    }

    /// List tracked files & submodules from HEAD tree directly via Gitoxide ODB.
    pub fn list_head_tree(path: &Path, max_entries: usize) -> Result<Vec<GixTreeEntry>, String> {
        let repo = gix::discover(path)
            .map_err(|e| format!("gix::discover failed for {}: {}", path.display(), e))?;
        let mut head = repo.head().map_err(|e| e.to_string())?;
        let commit = head
            .peel_to_commit_in_place()
            .map_err(|e| format!("HEAD has no commit: {}", e))?;
        let tree = commit.tree().map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        Self::walk_tree_recursive(&repo, &tree, "", &mut out, max_entries);
        Ok(out)
    }

    /// Read a blob's UTF-8 text at `HEAD:<rel_path>` directly from Gitoxide ODB without touching working tree.
    pub fn read_head_blob_utf8(path: &Path, rel_path: &str) -> Result<String, String> {
        let repo = gix::discover(path).map_err(|e| e.to_string())?;
        let mut head = repo.head().map_err(|e| e.to_string())?;
        let commit = head.peel_to_commit_in_place().map_err(|e| e.to_string())?;
        let mut tree = commit.tree().map_err(|e| e.to_string())?;
        let clean_rel = rel_path.trim_start_matches('/').replace('\\', "/");
        let entry = tree
            .peel_to_entry_by_path(&clean_rel)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Path '{}' not found in HEAD tree", clean_rel))?;
        let obj = entry.object().map_err(|e| e.to_string())?;
        String::from_utf8(obj.data.clone())
            .map_err(|_| format!("Blob '{}' is binary / non-UTF8", clean_rel))
    }

    fn count_tree_entries(
        repo: &gix::Repository,
        commit: &gix::Commit<'_>,
        cap: usize,
    ) -> (usize, usize) {
        let Ok(tree) = commit.tree() else {
            return (0, 0);
        };
        let mut entries = Vec::new();
        Self::walk_tree_recursive(repo, &tree, "", &mut entries, cap);
        let submodules = entries.iter().filter(|e| e.is_submodule).count();
        (entries.len(), submodules)
    }

    fn walk_tree_recursive(
        repo: &gix::Repository,
        tree: &gix::Tree<'_>,
        prefix: &str,
        out: &mut Vec<GixTreeEntry>,
        cap: usize,
    ) {
        if out.len() >= cap {
            return;
        }
        for entry_res in tree.iter() {
            if out.len() >= cap {
                break;
            }
            let Ok(entry) = entry_res else {
                continue;
            };
            let name = entry.filename().to_string();
            let rel = if prefix.is_empty() {
                name
            } else {
                format!("{}/{}", prefix, name)
            };
            let mode = entry.mode();
            if mode.is_tree() {
                if let Ok(obj) = repo.find_object(entry.oid())
                    && let Ok(sub_tree) = obj.try_into_tree()
                {
                    Self::walk_tree_recursive(repo, &sub_tree, &rel, out, cap);
                }
            } else {
                out.push(GixTreeEntry {
                    rel_path: rel,
                    oid: entry.oid().to_string(),
                    is_blob: mode.is_blob() || mode.is_link(),
                    is_submodule: mode.is_commit(),
                });
            }
        }
    }
}
