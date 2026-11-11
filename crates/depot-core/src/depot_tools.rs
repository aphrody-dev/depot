use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepotToolsInfo {
    pub root_path: Option<PathBuf>,
    pub gclient_found: bool,
    pub cipd_found: bool,
    pub autoninja_found: bool,
    pub siso_found: bool,
    pub fetch_found: bool,
    pub git_cache_dir: Option<PathBuf>,
}

pub struct DepotTools;

impl DepotTools {
    pub fn detect() -> DepotToolsInfo {
        let standard_paths = [
            PathBuf::from(r"C:\src\third_party\depot_tools"),
            PathBuf::from(r"C:\src\depot_tools"),
            PathBuf::from(r"C:\depot_tools"),
        ];

        let mut root_path = None;
        for path in &standard_paths {
            if path.exists() && path.join("gclient.py").exists() {
                root_path = Some(path.clone());
                break;
            }
        }

        // Check components
        let gclient_found = root_path
            .as_ref()
            .map(|r| r.join("gclient.bat").exists() || r.join("gclient").exists())
            .unwrap_or(false);

        let cipd_found = root_path
            .as_ref()
            .map(|r| r.join("cipd.bat").exists() || r.join("cipd").exists())
            .unwrap_or(false);

        let autoninja_found = root_path
            .as_ref()
            .map(|r| r.join("autoninja.bat").exists() || r.join("autoninja").exists())
            .unwrap_or(false);

        let siso_found = root_path
            .as_ref()
            .map(|r| r.join("siso.bat").exists() || r.join("siso.py").exists())
            .unwrap_or(false);

        let fetch_found = root_path
            .as_ref()
            .map(|r| r.join("fetch.bat").exists() || r.join("fetch.py").exists())
            .unwrap_or(false);

        // Git cache dir check (standard local appdata path)
        let cache_candidate = dirs_next_or_env();
        let git_cache_dir = cache_candidate.filter(|p| p.exists());

        DepotToolsInfo {
            root_path,
            gclient_found,
            cipd_found,
            autoninja_found,
            siso_found,
            fetch_found,
            git_cache_dir,
        }
    }

    pub fn run_passthrough(tool: &str, args: &[String]) -> Result<i32, String> {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(tool);
            c
        } else {
            Command::new(tool)
        };

        cmd.args(args);

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to execute {}: {}", tool, e))?;

        Ok(status.code().unwrap_or(1))
    }
}

fn dirs_next_or_env() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        let cache = PathBuf::from(appdata).join("depot_tools").join("git_cache");
        return Some(cache);
    }
    None
}
