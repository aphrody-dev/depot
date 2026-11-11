pub mod extractor;
pub mod languages;
pub mod query;
pub mod rope;
pub mod symbols;

pub use extractor::AstExtractor;
pub use languages::SupportedLanguage;
pub use query::{QueryCaptureItem, QueryRunner};
pub use rope::{RopeSession, apply_helix_rope_edits};
pub use symbols::{
    AstCall, AstFileSummary, AstImport, AstSymbol, Span, SymbolKind,
};

pub use ropey;
pub use tree_sitter;

use std::path::Path;

pub struct TreeSitterEngine;

impl TreeSitterEngine {
    pub fn detect_language(path: &Path) -> Option<SupportedLanguage> {
        SupportedLanguage::from_path(path)
    }

    pub fn parse_file(path: &Path, content: &str) -> Option<AstFileSummary> {
        let lang = Self::detect_language(path)?;
        let path_str = path.to_string_lossy().replace('\\', "/");
        AstExtractor::extract(lang, &path_str, content).ok()
    }

    pub fn parse_str(
        lang: SupportedLanguage,
        path_str: &str,
        content: &str,
    ) -> Result<AstFileSummary, String> {
        AstExtractor::extract(lang, path_str, content)
    }

    pub fn query_sexpr(
        lang: SupportedLanguage,
        content: &str,
        query_sexpr: &str,
    ) -> Result<Vec<QueryCaptureItem>, String> {
        QueryRunner::execute(lang, content, query_sexpr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_ast_extraction() {
        let code = r#"
pub struct EngineConfig {
    pub threads: usize,
}

pub enum EngineState {
    Starting,
    Running,
    Stopped,
}

pub trait Runnable {
    fn run(&self);
}

impl EngineConfig {
    pub fn new() -> Self {
        init_logging();
        Self { threads: 4 }
    }
}

fn init_logging() {
    println!("Init");
}
"#;
        let summary = TreeSitterEngine::parse_str(SupportedLanguage::Rust, "src/engine.rs", code).unwrap();
        assert_eq!(summary.language, "rust");
        assert!(summary.symbols.iter().any(|s| s.name == "EngineConfig" && s.kind == SymbolKind::Struct));
        assert!(summary.symbols.iter().any(|s| s.name == "EngineState" && s.kind == SymbolKind::Enum));
        assert!(summary.symbols.iter().any(|s| s.name == "Runnable" && s.kind == SymbolKind::Trait));
        assert!(summary.symbols.iter().any(|s| s.name == "new" && s.kind == SymbolKind::Method));
        assert!(summary.symbols.iter().any(|s| s.name == "init_logging" && s.kind == SymbolKind::Function));
        assert!(summary.calls.iter().any(|c| c.callee == "init_logging"));
    }

    #[test]
    fn test_typescript_ast_extraction() {
        let code = r#"
import { Router } from "express";

export interface UserProps {
    id: string;
    name: string;
}

export class UserService {
    public getUser(id: string): UserProps {
        validateId(id);
        return { id, name: "Alice" };
    }
}

function validateId(id: string) {}
"#;
        let summary = TreeSitterEngine::parse_str(SupportedLanguage::TypeScript, "src/user.ts", code).unwrap();
        assert_eq!(summary.language, "typescript");
        assert!(summary.symbols.iter().any(|s| s.name == "UserProps" && s.kind == SymbolKind::Interface));
        assert!(summary.symbols.iter().any(|s| s.name == "UserService" && s.kind == SymbolKind::Class));
        assert!(summary.symbols.iter().any(|s| s.name == "getUser" && s.kind == SymbolKind::Method));
        assert!(summary.symbols.iter().any(|s| s.name == "validateId" && s.kind == SymbolKind::Function));
        assert!(summary.imports.iter().any(|i| i.source_module == "express"));
    }

    #[test]
    fn test_python_ast_extraction() {
        let code = r#"
import os
from pathlib import Path

class DataProcessor:
    def __init__(self, path: str):
        self.path = path

    def process(self):
        self.load()

def standalone():
    pass
"#;
        let summary = TreeSitterEngine::parse_str(SupportedLanguage::Python, "processor.py", code).unwrap();
        assert_eq!(summary.language, "python");
        assert!(summary.symbols.iter().any(|s| s.name == "DataProcessor" && s.kind == SymbolKind::Class));
        assert!(summary.symbols.iter().any(|s| s.name == "__init__" && s.kind == SymbolKind::Method));
        assert!(summary.symbols.iter().any(|s| s.name == "process" && s.kind == SymbolKind::Method));
        assert!(summary.symbols.iter().any(|s| s.name == "standalone" && s.kind == SymbolKind::Function));
    }

    #[test]
    fn test_tree_sitter_query_sexpr() {
        let code = "fn compute_sum(a: i32, b: i32) -> i32 { a + b }";
        let query = "(function_item name: (identifier) @fn_name)";
        let results = TreeSitterEngine::query_sexpr(SupportedLanguage::Rust, code, query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].capture_name, "fn_name");
        assert_eq!(results[0].text, "compute_sum");
    }

    #[test]
    fn test_lua_ast_extraction() {
        let code = r#"
local bit32 = require("bit32")
local _ENV = { bit32 = bit32, table = table }

local function compute_hash(val)
    if val == 0 then
        goto skip_calc
    end
    do return bit32.bxor(val, 0x52) end
    ::skip_calc::
    return 0
end

function Player:level_up()
    self.level = self.level + 1
    compute_hash(self.level)
end
"#;
        let summary = TreeSitterEngine::parse_str(SupportedLanguage::Lua, "script.lua", code).unwrap();
        assert_eq!(summary.language, "lua");
        assert!(summary.symbols.iter().any(|s| s.name == "_ENV" && s.kind == SymbolKind::Variable));
        assert!(summary.symbols.iter().any(|s| s.name == "compute_hash" && s.kind == SymbolKind::Function));
        assert!(summary.symbols.iter().any(|s| s.name == "skip_calc" && s.kind == SymbolKind::Label));
        assert!(summary.symbols.iter().any(|s| s.name == "Player:level_up" && s.kind == SymbolKind::Method));
        assert!(summary.imports.iter().any(|i| i.source_module == "bit32"));
        assert!(summary.calls.iter().any(|c| c.callee == "compute_hash"));
    }
}
