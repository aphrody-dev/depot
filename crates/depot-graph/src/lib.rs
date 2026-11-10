pub mod export;
pub mod graph;
pub mod parser;
pub mod query;

pub use export::GraphExporter;
pub use graph::{CodeGraph, GraphEdge, GraphNode};
pub use parser::CodeParser;
pub use query::GraphQuery;

use std::path::Path;
use depot_core::SmartFilter;

pub fn build_graph_for_dir(root_dir: &Path, exclude_patterns: &[String]) -> CodeGraph {
    let walker = SmartFilter::build_walker(root_dir, true, true, exclude_patterns);

    let files: Vec<std::path::PathBuf> = walker
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.into_path())
        .collect();

    let mut graph = CodeGraph::new();

    // Parse each file
    for full_path in &files {
        let rel = match full_path.strip_prefix(root_dir) {
            Ok(r) => r,
            Err(_) => full_path.as_path(),
        };

        if let Ok(content) = std::fs::read_to_string(full_path) {
            CodeParser::parse_file(rel, &content, &mut graph);
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser_and_god_nodes() {
        let code = r#"
pub struct EngineCore;
pub struct RenderPipeline;

impl EngineCore {
    pub fn init() {}
}
"#;
        let mut graph = CodeGraph::new();
        CodeParser::parse_file(Path::new("src/engine.rs"), code, &mut graph);

        assert!(graph.nodes.iter().any(|n| n.label == "EngineCore"));
        assert!(graph.nodes.iter().any(|n| n.label == "RenderPipeline"));
        assert!(graph.nodes.iter().any(|n| n.label == "init()"));

        let god_nodes = graph.compute_degrees_and_god_nodes(5);
        assert!(!god_nodes.is_empty());
    }

    #[test]
    fn test_community_detection() {
        let mut graph = CodeGraph::new();
        graph.add_node(GraphNode {
            id: "a".to_string(),
            label: "A".to_string(),
            file_type: "struct".to_string(),
            source_file: "a.rs".to_string(),
            source_location: "L1".to_string(),
            community: 0,
            degree: None,
        });
        graph.add_node(GraphNode {
            id: "b".to_string(),
            label: "B".to_string(),
            file_type: "struct".to_string(),
            source_file: "b.rs".to_string(),
            source_location: "L1".to_string(),
            community: 0,
            degree: None,
        });
        graph.add_edge(GraphEdge {
            source: "a".to_string(),
            target: "b".to_string(),
            relation: "calls".to_string(),
        });

        let comms = graph.detect_communities();
        assert!(comms >= 1);
    }
}
