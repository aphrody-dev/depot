use std::path::Path;
use regex::Regex;
use crate::graph::{CodeGraph, GraphEdge, GraphNode};

pub struct CodeParser;

impl CodeParser {
    pub fn parse_file(path: &Path, content: &str, graph: &mut CodeGraph) {
        let rel_path_str = path.to_string_lossy().replace('\\', "/");
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let file_node_id = format!("file_{}", sanitize_id(&rel_path_str));

        graph.add_node(GraphNode {
            id: file_node_id.clone(),
            label: format!("{}.{}", file_stem, ext),
            file_type: "file".to_string(),
            source_file: rel_path_str.clone(),
            source_location: "L1".to_string(),
            community: 0,
            degree: None,
        });

        match ext {
            "rs" => Self::parse_rust(path, &rel_path_str, &file_node_id, content, graph),
            "ts" | "tsx" | "js" | "jsx" => {
                Self::parse_typescript(path, &rel_path_str, &file_node_id, content, graph)
            }
            "py" => Self::parse_python(path, &rel_path_str, &file_node_id, content, graph),
            "cpp" | "cc" | "cxx" | "h" | "hpp" | "cu" => {
                Self::parse_cpp(path, &rel_path_str, &file_node_id, content, graph)
            }
            "go" => Self::parse_go(path, &rel_path_str, &file_node_id, content, graph),
            _ => {}
        }
    }

    fn parse_rust(
        _path: &Path,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let struct_re = Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?struct\s+([A-Za-z0-9_]+)").unwrap();
        let enum_re = Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?enum\s+([A-Za-z0-9_]+)").unwrap();
        let trait_re = Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?trait\s+([A-Za-z0-9_]+)").unwrap();
        let fn_re = Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)").unwrap();
        let use_re = Regex::new(r"(?m)^\s*use\s+([^;]+);").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let l_num = line_num + 1;

            if let Some(caps) = struct_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("struct_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "struct".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = enum_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("enum_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "enum".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = trait_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("trait_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "trait".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = fn_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("fn_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: format!("{}()", name),
                    file_type: "function".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = use_re.captures(line) {
                let imported = caps[1].trim();
                let mod_name = imported.split("::").next().unwrap_or(imported);
                let import_id = format!("mod_{}", sanitize_id(mod_name));
                graph.add_node(GraphNode {
                    id: import_id.clone(),
                    label: mod_name.to_string(),
                    file_type: "import".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: import_id,
                    relation: "imports".to_string(),
                });
            }
        }
    }

    fn parse_typescript(
        _path: &Path,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let class_re = Regex::new(r"(?m)^\s*(?:export\s+)?(?:default\s+)?class\s+([A-Za-z0-9_]+)").unwrap();
        let interface_re = Regex::new(r"(?m)^\s*(?:export\s+)?interface\s+([A-Za-z0-9_]+)").unwrap();
        let fn_re = Regex::new(r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z0-9_]+)").unwrap();
        let import_re = Regex::new(r#"(?m)^\s*import\s+.*from\s+['"]([^'"]+)['"]"#).unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let l_num = line_num + 1;

            if let Some(caps) = class_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("class_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "class".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = interface_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("iface_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "interface".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = fn_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("fn_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: format!("{}()", name),
                    file_type: "function".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = import_re.captures(line) {
                let mod_name = &caps[1];
                let import_id = format!("import_{}", sanitize_id(mod_name));
                graph.add_node(GraphNode {
                    id: import_id.clone(),
                    label: mod_name.to_string(),
                    file_type: "import".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: import_id,
                    relation: "imports".to_string(),
                });
            }
        }
    }

    fn parse_python(
        _path: &Path,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let class_re = Regex::new(r"(?m)^\s*class\s+([A-Za-z0-9_]+)").unwrap();
        let fn_re = Regex::new(r"(?m)^\s*(?:async\s+)?def\s+([A-Za-z0-9_]+)").unwrap();
        let import_re = Regex::new(r"(?m)^\s*(?:from\s+([A-Za-z0-9_.]+)\s+import|import\s+([A-Za-z0-9_.]+))").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let l_num = line_num + 1;

            if let Some(caps) = class_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("class_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "class".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = fn_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("def_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: format!("{}()", name),
                    file_type: "function".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = import_re.captures(line) {
                let mod_name = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("");
                let import_id = format!("import_{}", sanitize_id(mod_name));
                graph.add_node(GraphNode {
                    id: import_id.clone(),
                    label: mod_name.to_string(),
                    file_type: "import".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: import_id,
                    relation: "imports".to_string(),
                });
            }
        }
    }

    fn parse_cpp(
        _path: &Path,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let class_re = Regex::new(r"(?m)^\s*(?:class|struct)\s+([A-Za-z0-9_]+)").unwrap();
        let inc_re = Regex::new(r#"(?m)^\s*#include\s+[<"]([^>"]+)[>"]"#).unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let l_num = line_num + 1;

            if let Some(caps) = class_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("cpp_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "class".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = inc_re.captures(line) {
                let header = &caps[1];
                let inc_id = format!("inc_{}", sanitize_id(header));
                graph.add_node(GraphNode {
                    id: inc_id.clone(),
                    label: header.to_string(),
                    file_type: "include".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: inc_id,
                    relation: "includes".to_string(),
                });
            }
        }
    }

    fn parse_go(
        _path: &Path,
        rel_str: &str,
        file_node_id: &str,
        content: &str,
        graph: &mut CodeGraph,
    ) {
        let type_re = Regex::new(r"(?m)^\s*type\s+([A-Za-z0-9_]+)\s+(?:struct|interface)").unwrap();
        let fn_re = Regex::new(r"(?m)^\s*func\s+(?:\([^)]+\)\s+)?([A-Za-z0-9_]+)\(").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let l_num = line_num + 1;

            if let Some(caps) = type_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("go_type_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: name.to_string(),
                    file_type: "type".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            } else if let Some(caps) = fn_re.captures(line) {
                let name = &caps[1];
                let sym_id = format!("go_fn_{}_{}", sanitize_id(rel_str), name);
                graph.add_node(GraphNode {
                    id: sym_id.clone(),
                    label: format!("{}()", name),
                    file_type: "function".to_string(),
                    source_file: rel_str.to_string(),
                    source_location: format!("L{}", l_num),
                    community: 0,
                    degree: None,
                });
                graph.add_edge(GraphEdge {
                    source: file_node_id.to_string(),
                    target: sym_id,
                    relation: "defines".to_string(),
                });
            }
        }
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
