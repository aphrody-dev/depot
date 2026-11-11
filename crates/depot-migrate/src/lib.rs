// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `depot-migrate`: Universal AST codemods and migrations engine targeting Bun v1.4.4 & Rust LTO.
//! Combines Tree-Sitter AST inspection, Helix Rope transactional rewrites, and package sanitization.

pub mod archetype;
pub mod manifest;
pub mod rules;
pub mod transformer;

pub use archetype::{ProjectArchetype, detect_archetype};
pub use manifest::{generate_bunfig, sanitize_package_json};
pub use rules::{CodemodRule, MIGRATION_RULES};
pub use transformer::{TransformFinding, TransformResult, transform_source};

use depot_treesitter::SupportedLanguage;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRunReport {
    pub root: String,
    pub archetype: ProjectArchetype,
    pub target_stack: String,
    pub files_scanned: usize,
    pub files_modified: usize,
    pub total_edits: usize,
    pub legacy_deps_removed: Vec<String>,
}

pub struct MigrateEngine;

impl MigrateEngine {
    pub fn run(root: &Path, dry_run: bool) -> Result<MigrationRunReport, String> {
        let archetype = detect_archetype(root);
        let target_stack = archetype.target_stack().to_string();

        let mut files_scanned = 0usize;
        let mut files_modified = 0usize;
        let mut total_edits = 0usize;
        let mut legacy_deps_removed = Vec::new();

        // Check package.json
        let pkg_path = root.join("package.json");
        if pkg_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&pkg_path) {
                if let Ok((sanitized, removed)) = sanitize_package_json(&raw) {
                    if !removed.is_empty() {
                        legacy_deps_removed = removed;
                        if !dry_run {
                            let _ = std::fs::write(&pkg_path, sanitized);
                        }
                    }
                }
            }
        }

        let walker = WalkBuilder::new(root)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            files_scanned += 1;

            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang = match ext {
                "ts" | "mts" | "cts" | "tsx" => SupportedLanguage::TypeScript,
                "js" | "mjs" | "cjs" | "jsx" => SupportedLanguage::JavaScript,
                "lua" => SupportedLanguage::Lua,
                _ => continue,
            };

            if let Ok(content) = std::fs::read_to_string(p) {
                let res = transform_source(lang, &content);
                if res.edits_count > 0 {
                    files_modified += 1;
                    total_edits += res.edits_count;
                    if !dry_run {
                        let _ = std::fs::write(p, res.transformed_code);
                    }
                }
            }
        }

        // Synthesize bunfig.toml if missing
        let bunfig_path = root.join("bunfig.toml");
        if !bunfig_path.exists() && !dry_run {
            let _ = std::fs::write(&bunfig_path, generate_bunfig());
        }

        Ok(MigrationRunReport {
            root: root.to_string_lossy().to_string(),
            archetype,
            target_stack,
            files_scanned,
            files_modified,
            total_edits,
            legacy_deps_removed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_sanitize_pkg() {
        let raw = r#"{
            "name": "my-legacy-app",
            "dependencies": {
                "axios": "^1.0.0",
                "lodash": "^4.0.0"
            }
        }"#;
        let (sanitized, removed) = sanitize_package_json(raw).unwrap();
        assert_eq!(removed, vec!["axios".to_string()]);
        assert!(!sanitized.contains("\"axios\""));
        assert!(sanitized.contains("\"lodash\""));
    }
}
