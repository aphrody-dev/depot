use std::path::Path;
use depot_core::JustEngine;
use depot_treesitter::{SupportedLanguage, TreeSitterEngine};

use crate::graph::{CodeGraph, EdgeProvenance, GraphEdge, GraphNode};

pub struct CodeParser;

impl CodeParser {
    pub fn parse_file(path: &Path, content: &str, graph: &mut CodeGraph) {
        let rel_path_str = path.to_string_lossy().replace('\\', "/");
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        let file_node_id = format!("file_{}", sanitize_id(&rel_path_str));

        let display_label = if ext.is_empty() {
            file_name.to_string()
        } else {
            format!("{}.{}", file_stem, ext)
        };

        graph.add_node(GraphNode {
            id: file_node_id.clone(),
            label: display_label,
            file_type: "file".to_string(),
            source_file: rel_path_str.clone(),
            source_location: "L1".to_string(),
            community: 0,
            degree: None,
            signature: None,
            provenance: "EXTRACTED".to_string(),
        });

        // 1. Justfile support
        if file_name.eq_ignore_ascii_case("justfile") {
            Self::parse_justfile(&rel_path_str, &file_node_id, content, graph);
            return;
        }

        // 2. Tree-sitter multi-language AST extraction
        if let Some(lang) = TreeSitterEngine::detect_language(path) {
            Self::parse_treesitter(lang, &rel_path_str, &file_node_id, content, graph);
        }
    }

    fn parse_justfile(
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let spec = JustEngine::parse_str(rel_str, content);
        for recipe in &spec.recipes {
                let rec_id = format!("recipe_{}_{}", sanitize_id(rel_str), recipe.name);
                let loc = format!("L{}", recipe.line_number);

                graph.add_node(GraphNode {
                    id: rec_id.clone(),
                    label: recipe.name.clone(),
                    file_type: "recipe".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: loc,
                    community: 0,
                    degree: None,
                    signature: recipe.doc.clone(),
                    provenance: "EXTRACTED".to_string(),
                });

                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: rec_id.clone(),
                    relation: "defines".to_string(),
                    provenance: EdgeProvenance::Extracted,
                });

                for dep in &recipe.dependencies {
                    let dep_id = format!("recipe_{}_{}", sanitize_id(rel_str), dep);
                    graph.add_edge(GraphEdge {
                        source: rec_id.clone(),
                        target: dep_id,
                        relation: "depends_on".to_string(),
                        provenance: EdgeProvenance::Extracted,
                    });
                }
            }
    }

    fn parse_treesitter(
        lang: SupportedLanguage,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let Ok(summary) = TreeSitterEngine::parse_str(lang, rel_str, content) else {
            return;
        };

        // Track symbol ids to map calls
        let mut symbol_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for sym in &summary.symbols {
            let sym_id = format!(
                "{}_{}_{}",
                sym.kind.as_str(),
                sanitize_id(rel_str),
                sanitize_id(&sym.name)
            );
            symbol_map.insert(sym.name.clone(), sym_id.clone());

            graph.add_node(GraphNode {
                id: sym_id.clone(),
                label: sym.name.clone(),
                file_type: sym.kind.as_str().to_string(),
                source_file: rel_str.to_string(),
                source_location: sym.span.source_location(),
                community: 0,
                degree: None,
                signature: sym.signature.clone(),
                provenance: "EXTRACTED".to_string(),
            });

            graph.add_edge(GraphEdge {
                source: file_node_id.to_string(),
                target: sym_id,
                relation: "defines".to_string(),
                provenance: EdgeProvenance::Extracted,
            });
        }

        // Imports
        for imp in &summary.imports {
            let mod_id = format!("mod_{}", sanitize_id(&imp.source_module));
            graph.add_node(GraphNode {
                id: mod_id.clone(),
                label: imp.source_module.clone(),
                file_type: "import".to_string(),
                source_file: rel_str.to_string(),
                source_location: imp.span.source_location(),
                community: 0,
                degree: None,
                signature: None,
                provenance: "EXTRACTED".to_string(),
            });

            graph.add_edge(GraphEdge {
                source: file_node_id.to_string(),
                target: mod_id,
                relation: "imports".to_string(),
                provenance: EdgeProvenance::Extracted,
            });
        }

        // Calls
        for call in &summary.calls {
            let caller_id = call
                .caller_scope
                .as_ref()
                .and_then(|scope| symbol_map.get(scope))
                .cloned()
                .unwrap_or_else(|| file_node_id.to_string());

            let target_id = symbol_map
                .get(&call.callee)
                .cloned()
                .unwrap_or_else(|| format!("fn_{}", sanitize_id(&call.callee)));

            // Add placeholder node if not defined
            if !graph.nodes.iter().any(|n| n.id == target_id) {
                graph.add_node(GraphNode {
                    id: target_id.clone(),
                    label: format!("{}()", call.callee),
                    file_type: "function".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: call.span.source_location(),
                    community: 0,
                    degree: None,
                    signature: None,
                    provenance: "INFERRED".to_string(),
                });
            }

            graph.add_edge(GraphEdge {
                source: caller_id,
                target: target_id,
                relation: "calls".to_string(),
                provenance: EdgeProvenance::Inferred,
            });
        }
    }
}

pub fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
