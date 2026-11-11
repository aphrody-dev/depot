use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Span {
    pub fn new(
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line,
            end_line,
            start_col,
            end_col,
            start_byte,
            end_byte,
        }
    }

    pub fn from_node(node: &tree_sitter::Node) -> Self {
        let start_point = node.start_position();
        let end_point = node.end_position();
        Self {
            start_line: start_point.row + 1,
            end_line: end_point.row + 1,
            start_col: start_point.column + 1,
            end_col: end_point.column + 1,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }

    pub fn source_location(&self) -> String {
        if self.start_line == self.end_line {
            format!("L{}", self.start_line)
        } else {
            format!("L{}-L{}", self.start_line, self.end_line)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Class,
    Enum,
    Trait,
    Interface,
    TypeAlias,
    Module,
    Constant,
    Variable,
    Label,
    Recipe,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Class => "class",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Interface => "interface",
            Self::TypeAlias => "type",
            Self::Module => "module",
            Self::Constant => "constant",
            Self::Variable => "variable",
            Self::Label => "label",
            Self::Recipe => "recipe",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstCall {
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_scope: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstImport {
    pub source_module: String,
    pub imported_symbols: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFileSummary {
    pub path: String,
    pub language: String,
    pub symbols: Vec<AstSymbol>,
    pub calls: Vec<AstCall>,
    pub imports: Vec<AstImport>,
    pub parse_errors: usize,
    pub line_count: usize,
    pub byte_count: usize,
}
