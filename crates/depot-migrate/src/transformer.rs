// SPDX-License-Identifier: Apache-2.0 OR MIT
//! AST-guided transformer using Tree-Sitter and Helix Rope transactional editing.

use depot_treesitter::{RopeSession, SupportedLanguage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformFinding {
    pub rule_id: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub original: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub findings: Vec<TransformFinding>,
    pub transformed_code: String,
    pub edits_count: usize,
}

/// Scans source text for legacy Node patterns and rewrites them using Helix Rope reverse-order edits.
pub fn transform_source(_language: SupportedLanguage, source: &str) -> TransformResult {
    let mut findings = Vec::new();

    // Check for require('fs') -> require('node:fs')
    let legacy_requires = [
        ("require('fs')", "require('node:fs')"),
        ("require(\"fs\")", "require(\"node:fs\")"),
        ("require('path')", "require('node:path')"),
        ("require(\"path\")", "require(\"node:path\")"),
        ("require('crypto')", "require('node:crypto')"),
        ("require(\"crypto\")", "require(\"node:crypto\")"),
        ("require('os')", "require('node:os')"),
        ("require(\"os\")", "require(\"node:os\")"),
        ("require('sqlite3')", "require('bun:sqlite')"),
        ("require(\"sqlite3\")", "require(\"bun:sqlite\")"),
        ("require('better-sqlite3')", "require('bun:sqlite')"),
        ("require(\"better-sqlite3\")", "require(\"bun:sqlite\")"),
    ];

    for &(legacy, rep) in &legacy_requires {
        let mut offset = 0;
        while let Some(pos) = source[offset..].find(legacy) {
            let start = offset + pos;
            let end = start + legacy.len();
            findings.push(TransformFinding {
                rule_id: "BUN-MODERNIZE".to_string(),
                start_byte: start,
                end_byte: end,
                original: legacy.to_string(),
                replacement: rep.to_string(),
            });
            offset = end;
        }
    }

    if findings.is_empty() {
        return TransformResult {
            findings: Vec::new(),
            transformed_code: source.to_string(),
            edits_count: 0,
        };
    }

    let mut session = RopeSession::new(source);
    for f in &findings {
        session.stage_replace(f.start_byte, f.end_byte, &f.replacement);
    }

    let count = findings.len();
    let transformed = session.commit();

    TransformResult {
        findings,
        transformed_code: transformed,
        edits_count: count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_modernize() {
        let code = "const fs = require('fs');\nconst db = require('sqlite3');";
        let res = transform_source(SupportedLanguage::JavaScript, code);
        assert_eq!(res.edits_count, 2);
        assert!(res.transformed_code.contains("require('node:fs')"));
        assert!(res.transformed_code.contains("require('bun:sqlite')"));
    }
}
