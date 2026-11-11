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

/// Scans source text for legacy Node or Lua 5.1 patterns and rewrites them using Helix Rope reverse-order edits.
pub fn transform_source(language: SupportedLanguage, source: &str) -> TransformResult {
    let mut findings = Vec::new();

    if language == SupportedLanguage::Lua {
        let lua_rules = [
            ("LUA52-BIT32", "bit.band(", "bit32.band("),
            ("LUA52-BIT32", "bit.bor(", "bit32.bor("),
            ("LUA52-BIT32", "bit.bxor(", "bit32.bxor("),
            ("LUA52-BIT32", "bit.bnot(", "bit32.bnot("),
            ("LUA52-BIT32", "bit.lshift(", "bit32.lshift("),
            ("LUA52-BIT32", "bit.rshift(", "bit32.rshift("),
            ("LUA52-BIT32", "bit.arshift(", "bit32.arshift("),
            ("LUA52-MATH-LOG10", "math.log10(", "math.log("),
            ("LUA52-LOADSTRING", "loadstring(", "load("),
        ];

        for &(rule_id, legacy, rep) in &lua_rules {
            let mut offset = 0;
            while let Some(pos) = source[offset..].find(legacy) {
                let start = offset + pos;
                let end = start + legacy.len();
                let prev_ok = start == 0 || {
                    let b = source.as_bytes()[start - 1];
                    !b.is_ascii_alphanumeric() && b != b'_' && b != b'.'
                };
                if prev_ok {
                    findings.push(TransformFinding {
                        rule_id: rule_id.to_string(),
                        start_byte: start,
                        end_byte: end,
                        original: legacy.to_string(),
                        replacement: rep.to_string(),
                    });
                }
                offset = end;
            }
        }

        // Global unpack( -> table.unpack( (only when not preceded by `.`, `:`, or identifier char)
        let legacy_unpack = "unpack(";
        let mut offset = 0;
        while let Some(pos) = source[offset..].find(legacy_unpack) {
            let start = offset + pos;
            let end = start + legacy_unpack.len();
            let prev_ok = start == 0 || {
                let b = source.as_bytes()[start - 1];
                !b.is_ascii_alphanumeric() && b != b'_' && b != b'.' && b != b':'
            };
            if prev_ok {
                findings.push(TransformFinding {
                    rule_id: "LUA52-TABLE-UNPACK".to_string(),
                    start_byte: start,
                    end_byte: end,
                    original: legacy_unpack.to_string(),
                    replacement: "table.unpack(".to_string(),
                });
            }
            offset = end;
        }
    } else {
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

    #[test]
    fn test_lua52_migration() {
        let code = "local a, b = unpack(items)\nlocal mask = bit.band(flags, 0xFF)\nlocal ok = table.unpack(other)\n";
        let res = transform_source(SupportedLanguage::Lua, code);
        assert_eq!(res.edits_count, 2);
        assert!(res.transformed_code.contains("local a, b = table.unpack(items)"));
        assert!(res.transformed_code.contains("local mask = bit32.band(flags, 0xFF)"));
        assert!(res.transformed_code.contains("local ok = table.unpack(other)"));
    }
}
