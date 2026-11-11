// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Package manifest (`package.json`) cleaner & `bunfig.toml` synthesis for Bun v1.4.4.

use serde_json::Value;

pub const OBSOLETE_NODE_PACKAGES: &[&str] = &[
    "ts-node",
    "nodemon",
    "dotenv",
    "rimraf",
    "cross-env",
    "node-fetch",
    "axios",
    "sqlite3",
    "better-sqlite3",
];

/// Sanitizes a `package.json` string, removing obsolete dependencies that Bun natively replaces.
pub fn sanitize_package_json(raw: &str) -> Result<(String, Vec<String>), String> {
    let mut val: Value = serde_json::from_str(raw).map_err(|e| format!("json parse error: {e}"))?;
    let mut removed = Vec::new();

    if let Some(deps) = val.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        for &pkg in OBSOLETE_NODE_PACKAGES {
            if deps.remove(pkg).is_some() {
                removed.push(pkg.to_string());
            }
        }
    }

    if let Some(dev_deps) = val.get_mut("devDependencies").and_then(|d| d.as_object_mut()) {
        for &pkg in OBSOLETE_NODE_PACKAGES {
            if dev_deps.remove(pkg).is_some() {
                removed.push(pkg.to_string());
            }
        }
    }

    let formatted = serde_json::to_string_pretty(&val).map_err(|e| format!("json format error: {e}"))?;
    Ok((formatted, removed))
}

/// Generates a canonical `bunfig.toml` for the repository.
pub fn generate_bunfig() -> &'static str {
    r#"# Canonical Bun v1.4.4 configuration synthesized by depot-migrate
[install]
lockfile = true
production = false

[test]
coverage = false

[run]
shell = "system"
"#
}
