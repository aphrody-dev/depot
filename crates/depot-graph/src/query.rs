use crate::graph::CodeGraph;

pub struct GraphQuery;

impl GraphQuery {
    pub fn query_symbol(graph: &CodeGraph, term: &str) -> Vec<String> {
        let term_lower = term.to_lowercase();
        let mut results = Vec::new();

        for node in &graph.nodes {
            if node.label.to_lowercase().contains(&term_lower) || node.id.to_lowercase().contains(&term_lower) {
                let mut info = format!(
                    "Node: {} [type: {}, loc: {}:{}]",
                    node.label, node.file_type, node.source_file, node.source_location
                );

                // Find incoming and outgoing edges
                let outgoing: Vec<&str> = graph
                    .edges
                    .iter()
                    .filter(|e| e.source == node.id)
                    .map(|e| e.target.as_str())
                    .collect();

                let incoming: Vec<&str> = graph
                    .edges
                    .iter()
                    .filter(|e| e.target == node.id)
                    .map(|e| e.source.as_str())
                    .collect();

                if !outgoing.is_empty() {
                    info.push_str(&format!("\n  -> connects to: {}", outgoing.join(", ")));
                }
                if !incoming.is_empty() {
                    info.push_str(&format!("\n  <- referenced by: {}", incoming.join(", ")));
                }

                results.push(info);
            }
        }

        results
    }
}
