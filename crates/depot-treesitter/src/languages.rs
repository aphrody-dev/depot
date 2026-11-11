use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
    Json,
    Bash,
    Html,
    Css,
    Lua,
}

impl SupportedLanguage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Json => "json",
            Self::Bash => "bash",
            Self::Html => "html",
            Self::Css => "css",
            Self::Lua => "lua",
        }
    }

    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Self::Json => tree_sitter_json::LANGUAGE.into(),
            Self::Bash => tree_sitter_bash::LANGUAGE.into(),
            Self::Html => tree_sitter_html::LANGUAGE.into(),
            Self::Css => tree_sitter_css::LANGUAGE.into(),
            Self::Lua => tree_sitter_lua::LANGUAGE.into(),
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "rs" => Some(Self::Rust),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" | "mjs" | "cjs" | "jsx" => Some(Self::JavaScript),
            "py" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" | "cu" | "cuh" => Some(Self::Cpp),
            "json" | "jsonc" => Some(Self::Json),
            "sh" | "bash" | "zsh" => Some(Self::Bash),
            "html" | "htm" => Some(Self::Html),
            "css" | "scss" => Some(Self::Css),
            "lua" => Some(Self::Lua),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::from_extension(ext)
        } else if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            match file_name {
                "Justfile" | "justfile" => None,
                "Dockerfile" | "dockerfile" => Some(Self::Bash),
                _ => None,
            }
        } else {
            None
        }
    }
}
