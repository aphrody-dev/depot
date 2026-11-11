pub mod export;
pub mod graph;
pub mod mcp;
pub mod parser;
pub mod query;

pub use export::GraphExporter;
pub use graph::{CodeGraph, EdgeProvenance, GraphEdge, GraphNode};
pub use mcp::McpServer;
pub use parser::CodeParser;
pub use query::{GraphQuery, QueryOptions};

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
    fn test_treesitter_and_justfile_graph() {
        let rs_code = r#"
pub struct EngineCore;
pub struct RenderPipeline;

impl EngineCore {
    pub fn init() {
        start_worker();
    }
}

fn start_worker() {}
"#;
        let mut graph = CodeGraph::new();
        CodeParser::parse_file(Path::new("src/engine.rs"), rs_code, &mut graph);

        let just_code = r#"
build:
    cargo build

test: build
    cargo test
"#;
        CodeParser::parse_file(Path::new("justfile"), just_code, &mut graph);

        assert!(graph.nodes.iter().any(|n| n.label == "EngineCore"));
        assert!(graph.nodes.iter().any(|n| n.label == "RenderPipeline"));
        assert!(graph.nodes.iter().any(|n| n.label == "init"));
        assert!(graph.nodes.iter().any(|n| n.label == "build"));
        assert!(graph.nodes.iter().any(|n| n.label == "test"));

        let god_nodes = graph.compute_degrees_and_god_nodes(5);
        assert!(!god_nodes.is_empty());

        let comms = graph.detect_communities();
        assert!(comms >= 1);

        // Test path
        let path_res = graph.shortest_path("EngineCore", "init");
        assert!(path_res.is_some());

        // Test explain
        let dossier = GraphQuery::explain(&graph, "EngineCore").unwrap();
        assert!(dossier.contains("Symbol Dossier: EngineCore"));

        // Test query
        let opts = QueryOptions::default();
        let query_res = GraphQuery::query(&graph, "Engine", &opts);
        assert!(query_res.contains("EngineCore"));
    }
}
