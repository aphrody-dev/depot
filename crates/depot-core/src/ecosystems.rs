use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepsPin {
    pub path: String,
    pub url: String,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AospProjectEntry {
    pub path: String,
    pub name: String,
    pub remote: String,
    pub revision: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelMaintainerSubsystem {
    pub title: String,
    pub maintainers: Vec<String>,
    pub status: String,
    pub trees: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemGovernanceStatus {
    pub deps_pin_count: usize,
    pub aosp_project_count: usize,
    pub kernel_subsystem_count: usize,
    pub patch_series_count: usize,
}

pub struct EcosystemManifests;

impl EcosystemManifests {
    /// Parse Chromium `DEPS` file pins (`'path': 'url@sha'`)
    pub fn parse_chromium_deps(content: &str) -> BTreeMap<String, DepsPin> {
        let mut pins = BTreeMap::new();
        let Ok(re) = Regex::new(r#"['"]([^'"]+)['"]\s*:\s*['"](https?://[^'"]+|git://[^'"]+)['"]"#)
        else {
            return pins;
        };

        for cap in re.captures_iter(content) {
            let Some(p_match) = cap.get(1) else { continue };
            let Some(u_match) = cap.get(2) else { continue };
            let path = p_match.as_str().to_string();
            let raw_spec = u_match.as_str();
            let (url, revision) = if let Some((u, rev)) = raw_spec.rsplit_once('@') {
                (u.to_string(), Some(rev.to_string()))
            } else {
                (raw_spec.to_string(), None)
            };
            pins.insert(
                path.clone(),
                DepsPin {
                    path,
                    url,
                    revision,
                },
            );
        }
        pins
    }

    /// Parse Android AOSP `.repo/manifests/default.xml` `<project ... />` entries
    pub fn parse_aosp_manifest(xml: &str) -> Vec<AospProjectEntry> {
        let mut out = Vec::new();
        let Ok(proj_re) = Regex::new(r#"<project\s+([^>]+?)/?>"#) else {
            return out;
        };
        let Ok(attr_re) = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)""#) else {
            return out;
        };

        let mut default_remote = "aosp".to_string();
        let mut default_rev = "main".to_string();
        if let Ok(def_re) = Regex::new(r#"<default\s+([^>]+?)/?>"#)
            && let Some(cap) = def_re.captures(xml)
            && let Some(attrs_m) = cap.get(1)
        {
            for ac in attr_re.captures_iter(attrs_m.as_str()) {
                let k = ac.get(1).map(|m| m.as_str()).unwrap_or("");
                let v = ac.get(2).map(|m| m.as_str()).unwrap_or("");
                if k == "remote" && !v.is_empty() {
                    default_remote = v.to_string();
                } else if k == "revision" && !v.is_empty() {
                    default_rev = v.to_string();
                }
            }
        }

        for cap in proj_re.captures_iter(xml) {
            let Some(attrs_m) = cap.get(1) else { continue };
            let mut attrs = BTreeMap::new();
            for ac in attr_re.captures_iter(attrs_m.as_str()) {
                if let (Some(k), Some(v)) = (ac.get(1), ac.get(2)) {
                    attrs.insert(k.as_str().to_string(), v.as_str().to_string());
                }
            }
            let Some(name) = attrs.get("name").cloned() else {
                continue;
            };
            let path = attrs.get("path").cloned().unwrap_or_else(|| name.clone());
            let remote = attrs
                .get("remote")
                .cloned()
                .unwrap_or_else(|| default_remote.clone());
            let revision = attrs
                .get("revision")
                .cloned()
                .unwrap_or_else(|| default_rev.clone());
            let groups = attrs
                .get("groups")
                .map(|g| {
                    g.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();

            out.push(AospProjectEntry {
                path,
                name,
                remote,
                revision,
                groups,
            });
        }
        out
    }

    /// Parse Linux Kernel `MAINTAINERS` subsystems (`M:`, `S:`, `T:`, `F:`)
    pub fn parse_kernel_maintainers(content: &str) -> Vec<KernelMaintainerSubsystem> {
        let mut subsystems = Vec::new();
        let normalized = content.replace("\r\n", "\n");

        for block in normalized.split("\n\n") {
            let lines: Vec<&str> = block
                .lines()
                .map(|l| l.trim_end())
                .filter(|l| !l.is_empty())
                .collect();
            if lines.len() < 2 {
                continue;
            }
            let title = lines[0];
            if title.starts_with('=')
                || title.starts_with('-')
                || title.starts_with("..")
                || title.contains(':')
                || title != title.to_ascii_uppercase()
            {
                continue;
            }

            let mut maintainers = Vec::new();
            let mut trees = Vec::new();
            let mut files = Vec::new();
            let mut status = "Maintained".to_string();

            for line in &lines[1..] {
                if let Some(rest) = line.strip_prefix("M:") {
                    maintainers.push(rest.trim().to_string());
                } else if let Some(rest) = line.strip_prefix("S:") {
                    status = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("T:") {
                    trees.push(rest.trim().to_string());
                } else if let Some(rest) = line.strip_prefix("F:") {
                    files.push(rest.trim().to_string());
                }
            }

            if !files.is_empty() {
                subsystems.push(KernelMaintainerSubsystem {
                    title: title.to_string(),
                    maintainers,
                    status,
                    trees,
                    files,
                });
            }
        }
        subsystems
    }

    /// Find matching maintainer subsystems for a relative file path (like `scripts/get_maintainer.pl`)
    pub fn lookup_maintainers_for_path(
        subsystems: &[KernelMaintainerSubsystem],
        rel_path: &str,
    ) -> Vec<KernelMaintainerSubsystem> {
        let norm = rel_path.replace('\\', "/");
        let mut matches = Vec::new();

        for sub in subsystems {
            if sub.title == "THE REST" {
                continue;
            }
            let hit = sub.files.iter().any(|pattern| {
                let pat = pattern.trim_end_matches('*');
                if pat.ends_with('/') {
                    norm.starts_with(pat) || format!("{}/", norm).starts_with(pat)
                } else {
                    norm == pat || norm.starts_with(&format!("{}/", pat))
                }
            });
            if hit {
                matches.push(sub.clone());
            }
        }

        if matches.is_empty() {
            for sub in subsystems {
                if sub.title == "THE REST" {
                    matches.push(sub.clone());
                }
            }
        }
        matches
    }

    /// Parse declarative Quilt / b4 `series` patch list
    pub fn parse_patch_series(content: &str) -> Vec<String> {
        content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.split_whitespace().next().unwrap_or(l).to_string())
            .collect()
    }

    /// Inspect full 4-pillar governance status at `root` (`C:\src`)
    pub fn inspect_root(root: &Path) -> EcosystemGovernanceStatus {
        let deps_pin_count = fs::read_to_string(root.join("DEPS"))
            .map(|c| Self::parse_chromium_deps(&c).len())
            .unwrap_or(0);

        let aosp_project_count = fs::read_to_string(root.join(".repo").join("manifests").join("default.xml"))
            .map(|c| Self::parse_aosp_manifest(&c).len())
            .unwrap_or(0);

        let kernel_subsystem_count = fs::read_to_string(root.join("MAINTAINERS"))
            .map(|c| Self::parse_kernel_maintainers(&c).len())
            .unwrap_or(0);

        let patch_series_count = fs::read_to_string(root.join("docs").join("patches").join("series"))
            .map(|c| Self::parse_patch_series(&c).len())
            .unwrap_or(0);

        EcosystemGovernanceStatus {
            deps_pin_count,
            aosp_project_count,
            kernel_subsystem_count,
            patch_series_count,
        }
    }
}
