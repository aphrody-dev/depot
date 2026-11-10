use std::fs;
use std::path::Path;
use crate::graph::CodeGraph;

pub struct GraphExporter;

impl GraphExporter {
    pub fn export_all(
        graph: &mut CodeGraph,
        out_dir: &Path,
        codebase_name: &str,
    ) -> Result<(), String> {
        fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

        let god_nodes = graph.compute_degrees_and_god_nodes(10);
        let num_communities = graph.detect_communities();

        // 1. graph.json (graphify compatible)
        let json_path = out_dir.join("graph.json");
        let json_data = serde_json::to_string_pretty(graph).map_err(|e| e.to_string())?;
        fs::write(&json_path, json_data).map_err(|e| e.to_string())?;

        // 2. GRAPH_REPORT.md (graphify style)
        let report_path = out_dir.join("GRAPH_REPORT.md");
        let report_content = Self::generate_report(graph, codebase_name, &god_nodes, num_communities);
        fs::write(&report_path, report_content).map_err(|e| e.to_string())?;

        // 3. ARCHITECTURE.mermaid
        let mermaid_path = out_dir.join("ARCHITECTURE.mermaid");
        let mermaid_content = Self::generate_mermaid(graph, 40);
        fs::write(&mermaid_path, mermaid_content).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn generate_report(
        graph: &CodeGraph,
        codebase_name: &str,
        god_nodes: &[(String, usize)],
        num_communities: usize,
    ) -> String {
        let mut r = String::new();
        r.push_str(&format!("# Graph Report - {}\n\n", codebase_name));

        r.push_str("## Corpus Check\n");
        r.push_str(&format!(
            "- {} nodes · {} edges · {} detected communities\n",
            graph.nodes.len(),
            graph.edges.len(),
            num_communities
        ));
        r.push_str("- Extraction: 100% DETERMINISTIC AST PARSING (Depot Native Engine)\n\n");

        r.push_str("## God Nodes (Most connected core abstractions)\n");
        for (idx, (node_id, degree)) in god_nodes.iter().enumerate() {
            let label = graph
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .map(|n| n.label.as_str())
                .unwrap_or(node_id.as_str());
            r.push_str(&format!("{}. `{}` - {} edges\n", idx + 1, label, degree));
        }

        r.push_str("\n## Sample Key Structural Connections\n");
        for edge in graph.edges.iter().take(15) {
            let src = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.source)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.source);
            let tgt = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.target)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.target);
            r.push_str(&format!("- `{}` --{}--> `{}`\n", src, edge.relation, tgt));
        }

        r.push_str("\n## Architecture Diagram (Mermaid)\n\n");
        r.push_str("```mermaid\n");
        r.push_str(&Self::generate_mermaid(graph, 25));
        r.push_str("\n```\n");

        r
    }

    pub fn generate_mermaid(graph: &CodeGraph, limit_nodes: usize) -> String {
        let mut m = String::new();
        m.push_str("flowchart TD\n");

        let visible_nodes: Vec<&crate::graph::GraphNode> =
            graph.nodes.iter().take(limit_nodes).collect();
        let visible_ids: std::collections::HashSet<&str> =
            visible_nodes.iter().map(|n| n.id.as_str()).collect();

        for node in &visible_nodes {
            let clean_label = node.label.replace('"', "").replace('(', "").replace(')', "");
            m.push_str(&format!("    {}[\"{}\"]\n", node.id, clean_label));
        }

        for edge in &graph.edges {
            if visible_ids.contains(edge.source.as_str()) && visible_ids.contains(edge.target.as_str()) {
                m.push_str(&format!(
                    "    {} -->|{}| {}\n",
                    edge.source, edge.relation, edge.target
                ));
            }
        }

        m
    }
}
