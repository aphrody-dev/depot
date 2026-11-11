//! Secure in-memory `.env` and environment vault loader (`C:\src\.env`).
//!
//! Reads tokens dynamically at runtime without ever logging or persisting raw credentials.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct VaultEnv;

impl VaultEnv {
    /// Load key/value pairs from `C:\src\.env` (if present) merged with process environment variables.
    pub fn load_merged() -> HashMap<String, String> {
        let mut map = HashMap::new();
        for candidate in [
            PathBuf::from(r"C:\src\.env"),
            PathBuf::from(".env"),
        ] {
            if candidate.is_file() {
                Self::load_file_into(&candidate, &mut map);
                break;
            }
        }
        for (k, v) in std::env::vars() {
            if !v.is_empty() {
                map.insert(k, v);
            }
        }
        map
    }

    pub fn get_first(keys: &[&str]) -> Option<String> {
        let map = Self::load_merged();
        for &k in keys {
            if let Some(val) = map.get(k)
                && !val.trim().is_empty()
            {
                return Some(val.trim().to_string());
            }
        }
        None
    }

    fn load_file_into(path: &Path, out: &mut HashMap<String, String>) {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return;
        };
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let stripped = trimmed.strip_prefix("export ").unwrap_or(trimmed);
            if let Some((k, v)) = stripped.split_once('=') {
                let key = k.trim().to_string();
                let val = v
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string();
                if !key.is_empty() && !val.is_empty() {
                    out.insert(key, val);
                }
            }
        }
    }
}
