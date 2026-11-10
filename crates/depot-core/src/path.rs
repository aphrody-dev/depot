use std::path::Path;

/// Normalizes file paths to canonical sovereign POSIX/GNU format with forward slashes.
pub fn normalize_posix(path: &Path) -> String {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => continue,
            std::path::Component::ParentDir => {
                if let Some(last) = components.last() {
                    if *last != ".." {
                        components.pop();
                        continue;
                    }
                }
                components.push("..");
            }
            std::path::Component::Normal(c) => {
                components.push(c.to_str().unwrap_or(""));
            }
            std::path::Component::RootDir => {
                components.clear();
                components.push("");
            }
            std::path::Component::Prefix(p) => {
                components.push(p.as_os_str().to_str().unwrap_or(""));
            }
        }
    }
    let joined = components.join("/");
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}
