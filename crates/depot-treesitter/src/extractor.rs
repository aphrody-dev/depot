use tree_sitter::{Node, Parser};

use crate::languages::SupportedLanguage;
use crate::symbols::{
    AstCall, AstFileSummary, AstImport, AstSymbol, Span, SymbolKind,
};

pub struct AstExtractor;

impl AstExtractor {
    pub fn extract(
        lang: SupportedLanguage,
        path_str: &str,
        source: &str,
    ) -> Result<AstFileSummary, String> {
        let ts_lang = lang.tree_sitter_language();
        let mut parser = Parser::new();
        parser
            .set_language(&ts_lang)
            .map_err(|e| format!("Failed to set language for {}: {:?}", lang.name(), e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| format!("Failed to parse {}", path_str))?;

        let root = tree.root_node();
        let bytes = source.as_bytes();
        let line_count = source.lines().count();
        let byte_count = bytes.len();

        let mut symbols = Vec::new();
        let mut calls = Vec::new();
        let mut imports = Vec::new();
        let mut errors = 0;

        let mut scope_stack: Vec<String> = Vec::new();

        walk_node(
            root,
            lang,
            bytes,
            &mut scope_stack,
            &mut symbols,
            &mut calls,
            &mut imports,
            &mut errors,
        );

        Ok(AstFileSummary {
            path: path_str.to_string(),
            language: lang.name().to_string(),
            symbols,
            calls,
            imports,
            parse_errors: errors,
            line_count,
            byte_count,
        })
    }
}

fn node_text<'a>(node: &Node<'a>, bytes: &'a [u8]) -> &'a str {
    node.utf8_text(bytes).unwrap_or("").trim()
}

fn extract_signature(node: &Node, bytes: &[u8]) -> Option<String> {
    let full = node.utf8_text(bytes).ok()?;
    let first_line = full.lines().next()?.trim();
    if let Some(pos) = first_line.find('{') {
        Some(first_line[..pos].trim().to_string())
    } else {
        Some(first_line.to_string())
    }
}

fn walk_node(
    node: Node,
    lang: SupportedLanguage,
    bytes: &[u8],
    scope_stack: &mut Vec<String>,
    symbols: &mut Vec<AstSymbol>,
    calls: &mut Vec<AstCall>,
    imports: &mut Vec<AstImport>,
    errors: &mut usize,
) {
    if node.is_error() || node.is_missing() {
        *errors += 1;
    }

    let kind = node.kind();
    let current_scope = scope_stack.last().cloned();
    let mut pushed_scope: Option<String> = None;

    match lang {
        SupportedLanguage::Rust => {
            match kind {
                "struct_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Struct,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "enum_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Enum,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "trait_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Trait,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "function_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        let sym_kind = if current_scope.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: sym_kind,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "impl_item" => {
                    if let Some(type_node) = node.child_by_field_name("type") {
                        let type_name = node_text(&type_node, bytes).to_string();
                        let scope_name = if let Some(trait_node) = node.child_by_field_name("trait") {
                            format!("{} for {}", node_text(&trait_node, bytes), type_name)
                        } else {
                            type_name
                        };
                        pushed_scope = Some(scope_name);
                    }
                }
                "mod_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Module,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "use_declaration" => {
                    let text = node_text(&node, bytes);
                    let clean = text.trim_start_matches("pub ")
                        .trim_start_matches("use ")
                        .trim_end_matches(';')
                        .trim();
                    let parts: Vec<&str> = clean.split("::").collect();
                    let module = parts.first().unwrap_or(&clean).to_string();
                    let imported = parts.last().unwrap_or(&clean).to_string();
                    imports.push(AstImport {
                        source_module: module,
                        imported_symbols: vec![imported],
                        span: Span::from_node(&node),
                    });
                }
                "call_expression" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee = node_text(&func_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::TypeScript | SupportedLanguage::Tsx | SupportedLanguage::JavaScript => {
            match kind {
                "class_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Class,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Interface,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "type_alias_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name,
                            kind: SymbolKind::TypeAlias,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                    }
                }
                "enum_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name,
                            kind: SymbolKind::Enum,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                    }
                }
                "function_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "method_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Method,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "import_statement" => {
                    let mut src_module = String::new();
                    if let Some(src_node) = node.child_by_field_name("source") {
                        src_module = node_text(&src_node, bytes).trim_matches(|c| c == '\'' || c == '"').to_string();
                    }
                    let mut imported = Vec::new();
                    if let Some(clause) = node.children(&mut node.walk()).find(|c| c.kind() == "import_clause") {
                        imported.push(node_text(&clause, bytes).to_string());
                    }
                    imports.push(AstImport {
                        source_module: src_module,
                        imported_symbols: imported,
                        span: Span::from_node(&node),
                    });
                }
                "call_expression" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee = node_text(&func_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::Python => {
            match kind {
                "class_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Class,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        let sym_kind = if current_scope.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: sym_kind,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "import_statement" => {
                    let text = node_text(&node, bytes).to_string();
                    imports.push(AstImport {
                        source_module: text.replace("import ", ""),
                        imported_symbols: Vec::new(),
                        span: Span::from_node(&node),
                    });
                }
                "import_from_statement" => {
                    if let Some(mod_node) = node.child_by_field_name("module_name") {
                        let mod_name = node_text(&mod_node, bytes).to_string();
                        imports.push(AstImport {
                            source_module: mod_name,
                            imported_symbols: Vec::new(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                "call" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee = node_text(&func_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::Go => {
            match kind {
                "type_declaration" => {
                    for i in 0..node.child_count() {
                        let child = node.child(i).unwrap();
                        if child.kind() == "type_spec" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = node_text(&name_node, bytes).to_string();
                                symbols.push(AstSymbol {
                                    name: name.clone(),
                                    kind: SymbolKind::Struct,
                                    span: Span::from_node(&child),
                                    signature: extract_signature(&child, bytes),
                                    doc: None,
                                    parent_scope: current_scope.clone(),
                                });
                            }
                        }
                    }
                }
                "function_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "method_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Method,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "import_declaration" => {
                    let text = node_text(&node, bytes);
                    imports.push(AstImport {
                        source_module: text.to_string(),
                        imported_symbols: Vec::new(),
                        span: Span::from_node(&node),
                    });
                }
                "call_expression" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee = node_text(&func_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::C | SupportedLanguage::Cpp => {
            match kind {
                "struct_specifier" | "class_specifier" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        let sym_kind = if kind == "class_specifier" {
                            SymbolKind::Class
                        } else {
                            SymbolKind::Struct
                        };
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: sym_kind,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "function_definition" => {
                    if let Some(decl_node) = node.child_by_field_name("declarator") {
                        let name = node_text(&decl_node, bytes);
                        // extract function identifier from declarator
                        let clean_name = name.split('(').next().unwrap_or(name).trim();
                        symbols.push(AstSymbol {
                            name: clean_name.to_string(),
                            kind: SymbolKind::Function,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(clean_name.to_string());
                    }
                }
                "preproc_include" => {
                    let text = node_text(&node, bytes);
                    let clean = text.replace("#include", "").trim().to_string();
                    imports.push(AstImport {
                        source_module: clean,
                        imported_symbols: Vec::new(),
                        span: Span::from_node(&node),
                    });
                }
                "call_expression" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee = node_text(&func_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::Bash => {
            match kind {
                "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "command" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let callee = node_text(&name_node, bytes).to_string();
                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        SupportedLanguage::Json => {
            if kind == "pair" {
                if let Some(key_node) = node.child_by_field_name("key") {
                    let key = node_text(&key_node, bytes).trim_matches('"').to_string();
                    symbols.push(AstSymbol {
                        name: key,
                        kind: SymbolKind::Constant,
                        span: Span::from_node(&node),
                        signature: extract_signature(&node, bytes),
                        doc: None,
                        parent_scope: current_scope.clone(),
                    });
                }
            }
        }
        SupportedLanguage::Lua => {
            match kind {
                "function_declaration" | "local_function_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = node_text(&name_node, bytes).to_string();
                        let kind = if name.contains(':') {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        symbols.push(AstSymbol {
                            name: name.clone(),
                            kind,
                            span: Span::from_node(&node),
                            signature: extract_signature(&node, bytes),
                            doc: None,
                            parent_scope: current_scope.clone(),
                        });
                        pushed_scope = Some(name);
                    }
                }
                "function_call" => {
                    let callee = if let Some(name_node) = node.child_by_field_name("name") {
                        node_text(&name_node, bytes).to_string()
                    } else if let Some(prefix_node) = node.child_by_field_name("prefix") {
                        node_text(&prefix_node, bytes).to_string()
                    } else {
                        String::new()
                    };

                    if !callee.is_empty() {
                        if callee == "require" || callee == "include" {
                            if let Some(args) = node.child_by_field_name("arguments") {
                                let arg_text = node_text(&args, bytes);
                                let clean = arg_text.trim_matches(|c| c == '(' || c == ')' || c == '"' || c == '\'');
                                imports.push(AstImport {
                                    source_module: clean.to_string(),
                                    imported_symbols: Vec::new(),
                                    span: Span::from_node(&node),
                                });
                            }
                        }

                        calls.push(AstCall {
                            callee,
                            caller_scope: current_scope.clone(),
                            span: Span::from_node(&node),
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    if let Some(ref scope) = pushed_scope {
        scope_stack.push(scope.clone());
    }

    // Recurse children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_node(
            child,
            lang,
            bytes,
            scope_stack,
            symbols,
            calls,
            imports,
            errors,
        );
    }

    if pushed_scope.is_some() {
        scope_stack.pop();
    }
}
