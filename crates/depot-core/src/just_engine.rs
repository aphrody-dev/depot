//! Native `just` (`casey/just`) parser, recipe DAG analyzer, and task runner.
//!
//! Parses `justfile` / `.justfile` / `Justfile` recipes, attributes (`[group(...)]`, `[windows]`,
//! `[linux]`, `[private]`, `[no-cd]`), variables, aliases, doc comments, parameters, and
//! recipe-to-recipe dependency edges so `just` tasks become first-class nodes in `depot-graph`
//! (`graphify`) and can be inspected or executed directly via `depot just`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustParam {
    pub name: String,
    pub default_value: Option<String>,
    pub variadic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustRecipe {
    pub name: String,
    pub doc: Option<String>,
    pub group: Option<String>,
    pub attributes: Vec<String>,
    pub params: Vec<JustParam>,
    pub dependencies: Vec<String>,
    pub body_lines: Vec<String>,
    pub line_number: usize,
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustfileSpec {
    pub path: String,
    pub settings: BTreeMap<String, String>,
    pub variables: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, String>,
    pub imports: Vec<String>,
    pub modules: Vec<String>,
    pub recipes: Vec<JustRecipe>,
}

pub struct JustEngine;

impl JustEngine {
    /// Locate a `justfile` inside `dir` (or if `dir` itself is a file, return it).
    pub fn find_justfile(dir_or_file: &Path) -> Option<PathBuf> {
        if dir_or_file.is_file() {
            return Some(dir_or_file.to_path_buf());
        }
        for candidate in ["justfile", "Justfile", ".justfile"] {
            let p = dir_or_file.join(candidate);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    /// Detect whether `just` binary is available on the system.
    pub fn detect_binary() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from(r"C:\Users\aphro\.local\bin\just.exe"),
            PathBuf::from(r"C:\Users\aphro\.cargo\bin\just.exe"),
        ];
        for c in &candidates {
            if c.is_file() {
                return Some(c.clone());
            }
        }
        let check = if cfg!(windows) {
            Command::new("where").arg("just").output()
        } else {
            Command::new("which").arg("just").output()
        };
        if let Ok(out) = check
            && out.status.success()
            && let Some(first) = String::from_utf8_lossy(&out.stdout).lines().next()
        {
            let p = PathBuf::from(first.trim());
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    /// Parse a `justfile` string into a full `JustfileSpec` AST including recipe DAG edges.
    pub fn parse_str(source_path: &str, content: &str) -> JustfileSpec {
        let set_re = Regex::new(r"^set\s+([A-Za-z0-9_-]+)(?:\s*:=\s*(.+))?$").unwrap();
        let var_re = Regex::new(r"^(?:export\s+)?([A-Za-z_][A-Za-z0-9_-]*)\s*:=\s*(.+)$").unwrap();
        let alias_re =
            Regex::new(r"^alias\s+([A-Za-z_][A-Za-z0-9_-]*)\s*:=\s*([A-Za-z_][A-Za-z0-9_-]*)$")
                .unwrap();
        let import_re = Regex::new(r#"^import\??\s+['"]?([^'"]+)['"]?$"#).unwrap();
        let mod_re = Regex::new(r#"^mod\??\s+([A-Za-z_][A-Za-z0-9_-]*)(?:\s+['"]?([^'"]+)['"]?)?$"#).unwrap();
        let attr_re = Regex::new(r"^\[([^\]]+)\]$").unwrap();

        let mut settings = BTreeMap::new();
        let mut variables = BTreeMap::new();
        let mut aliases = BTreeMap::new();
        let mut imports = Vec::new();
        let mut modules = Vec::new();
        let mut recipes: Vec<JustRecipe> = Vec::new();

        let mut pending_comments: Vec<String> = Vec::new();
        let mut pending_attrs: Vec<String> = Vec::new();
        let mut current_recipe_idx: Option<usize> = None;

        for (idx, raw_line) in content.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed_end = raw_line.trim_end();

            // Indented line belongs to current recipe body
            if (trimmed_end.starts_with(' ') || trimmed_end.starts_with('\t'))
                && let Some(r_idx) = current_recipe_idx
            {
                recipes[r_idx].body_lines.push(trimmed_end.trim().to_string());
                continue;
            }

            let trimmed = trimmed_end.trim();
            if trimmed.is_empty() {
                current_recipe_idx = None;
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            if let Some(comment) = trimmed.strip_prefix('#') {
                current_recipe_idx = None;
                pending_comments.push(comment.trim().to_string());
                continue;
            }

            current_recipe_idx = None;

            if let Some(caps) = attr_re.captures(trimmed) {
                for part in caps[1].split(',') {
                    let attr = part.trim();
                    if !attr.is_empty() {
                        pending_attrs.push(attr.to_string());
                    }
                }
                continue;
            }

            if let Some(caps) = set_re.captures(trimmed) {
                let key = caps[1].to_string();
                let val = caps
                    .get(2)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_else(|| "true".to_string());
                settings.insert(key, val);
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            if let Some(caps) = alias_re.captures(trimmed) {
                aliases.insert(caps[1].to_string(), caps[2].to_string());
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            if let Some(caps) = import_re.captures(trimmed) {
                imports.push(caps[1].trim().to_string());
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            if let Some(caps) = mod_re.captures(trimmed) {
                modules.push(caps[1].trim().to_string());
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            if let Some(caps) = var_re.captures(trimmed) {
                variables.insert(caps[1].to_string(), caps[2].trim().to_string());
                pending_comments.clear();
                pending_attrs.clear();
                continue;
            }

            // Recipe matching: name [params]: [deps]
            // Skip variable assignments (which contain :=)
            if !trimmed.contains(":=") && let Some((left, raw_deps)) = trimmed.split_once(':') {
                let left_clean = left.trim().strip_prefix('@').unwrap_or(left.trim());
                let mut parts = left_clean.split_whitespace();
                if let Some(name) = parts.next()
                    && (name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false))
                {
                    let raw_params = left_clean[name.len()..].trim();
                    let params = Self::parse_params(raw_params);
                    let dependencies = Self::parse_deps(raw_deps.trim());

                    let mut doc = if pending_comments.is_empty() {
                        None
                    } else {
                        Some(pending_comments.join(" "))
                    };

                    let mut group = None;
                    let mut is_private = name.starts_with('_');
                    for attr in &pending_attrs {
                        if attr == "private" {
                            is_private = true;
                        } else if let Some(inner) = attr
                            .strip_prefix("group(")
                            .and_then(|s| s.strip_suffix(')'))
                        {
                            group = Some(inner.trim_matches(|c| c == '\'' || c == '"').to_string());
                        } else if let Some(inner) = attr
                            .strip_prefix("doc(")
                            .and_then(|s| s.strip_suffix(')'))
                        {
                            doc = Some(inner.trim_matches(|c| c == '\'' || c == '"').to_string());
                        }
                    }

                    recipes.push(JustRecipe {
                        name: name.to_string(),
                        doc,
                        group,
                        attributes: std::mem::take(&mut pending_attrs),
                        params,
                        dependencies,
                        body_lines: Vec::new(),
                        line_number: line_no,
                        is_private,
                    });
                    pending_comments.clear();
                    current_recipe_idx = Some(recipes.len() - 1);
                    continue;
                }
            }

            pending_comments.clear();
            pending_attrs.clear();
        }

        JustfileSpec {
            path: source_path.replace('\\', "/"),
            settings,
            variables,
            aliases,
            imports,
            modules,
            recipes,
        }
    }

    pub fn parse_file(path: &Path) -> Result<JustfileSpec, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed reading {}: {}", path.display(), e))?;
        Ok(Self::parse_str(&crate::path::normalize_posix(path), &content))
    }

    /// Execute `just` with the given arguments inside `working_dir`.
    pub fn run_just(working_dir: &Path, args: &[String]) -> Result<i32, String> {
        let bin = Self::detect_binary().unwrap_or_else(|| PathBuf::from("just"));
        let status = Command::new(&bin)
            .current_dir(working_dir)
            .args(args)
            .status()
            .map_err(|e| format!("Failed to invoke `{}`: {}", bin.display(), e))?;
        Ok(status.code().unwrap_or(1))
    }

    fn parse_params(raw: &str) -> Vec<JustParam> {
        if raw.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for tok in raw.split_whitespace() {
            let (variadic, rest) = if let Some(s) = tok.strip_prefix('*') {
                (Some("*".to_string()), s)
            } else if let Some(s) = tok.strip_prefix('+') {
                (Some("+".to_string()), s)
            } else {
                (None, tok)
            };
            let rest = rest.trim_start_matches('$');
            if let Some((k, v)) = rest.split_once('=') {
                out.push(JustParam {
                    name: k.to_string(),
                    default_value: Some(v.trim_matches(|c| c == '\'' || c == '"').to_string()),
                    variadic,
                });
            } else if !rest.is_empty() {
                out.push(JustParam {
                    name: rest.to_string(),
                    default_value: None,
                    variadic,
                });
            }
        }
        out
    }

    fn parse_deps(raw: &str) -> Vec<String> {
        let cleaned = raw.split('#').next().unwrap_or("").trim();
        if cleaned.is_empty() {
            return Vec::new();
        }
        let mut deps = Vec::new();
        // Handle both simple `dep1 dep2` and parameterized `(dep arg)` dependencies
        let mut current = String::new();
        let mut paren_depth = 0usize;
        for ch in cleaned.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                }
                ')' => {
                    paren_depth = paren_depth.saturating_sub(1);
                    if paren_depth == 0 {
                        if let Some(first) = current.split_whitespace().next()
                            && first != "&&"
                        {
                            deps.push(first.trim_start_matches('@').to_string());
                        }
                        current.clear();
                    }
                }
                c if c.is_whitespace() && paren_depth == 0 => {
                    let tok = current.trim();
                    if !tok.is_empty() && tok != "&&" {
                        deps.push(tok.trim_start_matches('@').to_string());
                    }
                    current.clear();
                }
                c => current.push(c),
            }
        }
        let tok = current.trim();
        if !tok.is_empty() && tok != "&&" {
            deps.push(tok.split_whitespace().next().unwrap_or(tok).trim_start_matches('@').to_string());
        }
        deps
    }
}
