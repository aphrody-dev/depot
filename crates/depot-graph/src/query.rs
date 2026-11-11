use std::collections::{HashSet, VecDeque};
use crate::graph::CodeGraph;

pub struct GraphQuery;

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub dfs: bool,
    pub max_tokens: usize,
    pub max_depth: usize,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            dfs: false,
            max_tokens: 2000,
            max_depth: 3,
        }
    }
}

impl GraphQuery {
    /// Explain a node in complete detail (symbol dossier).
    pub fn explain(graph: &CodeGraph, term: &str) -> Result<String, String> {
        let node = graph
            .find_node_fuzzy(term)
            .ok_or_else(|| format!("Symbol '{}' not found in graph", term))?;

        let mut out = String::new();
        out.push_str(&format!("# Symbol Dossier: {}\n\n", node.label));
        out.push_str(&format!("- **ID:** `{}`\n", node.id));
        out.push_str(&format!("- **Type:** `{}`\n", node.file_type));
        out.push_str(&format!("- **Location:** `{}:{}`\n", node.source_file, node.source_location));
        out.push_str(&format!("- **Provenance:** `{}`\n", node.provenance));
        out.push_str(&format!("- **Community:** {}\n", node.community));
        if let Some(deg) = node.degree {
            out.push_str(&format!("- **Total Degree:** {}\n", deg));
        }
        if let Some(ref sig) = node.signature {
            out.push_str(&format!("- **Signature / Doc:** `{}`\n", sig));
        }

        // Incoming relations (references, callers, importers)
        let incoming: Vec<&crate::graph::GraphEdge> = graph
            .edges
            .iter()
            .filter(|e| e.target == node.id)
            .collect();

        out.push_str("\n### Incoming Connections (Referenced by)\n");
        if incoming.is_empty() {
            out.push_str("- None (root or entry point)\n");
        } else {
            for e in incoming {
                let src_node = graph.nodes.iter().find(|n| n.id == e.source);
                let label = src_node.map(|n| n.label.as_str()).unwrap_or(&e.source);
                out.push_str(&format!(
                    "- `{}` --[{}]--> `{}` ({:?})\n",
                    label, e.relation, node.label, e.provenance
                ));
            }
        }

        // Outgoing relations (callees, dependencies, children)
        let outgoing: Vec<&crate::graph::GraphEdge> = graph
            .edges
            .iter()
            .filter(|e| e.source == node.id)
            .collect();

        out.push_str("\n### Outgoing Connections (Connects to)\n");
        if outgoing.is_empty() {
            out.push_str("- None (leaf node)\n");
        } else {
            for e in outgoing {
                let tgt_node = graph.nodes.iter().find(|n| n.id == e.target);
                let label = tgt_node.map(|n| n.label.as_str()).unwrap_or(&e.target);
                out.push_str(&format!(
                    "- `{}` --[{}]--> `{}` ({:?})\n",
                    node.label, e.relation, label, e.provenance
                ));
            }
        }

        // Community peers
        let peers: Vec<&str> = graph
            .nodes
            .iter()
            .filter(|n| n.community == node.community && n.id != node.id)
            .map(|n| n.label.as_str())
            .take(10)
            .collect();

        if !peers.is_empty() {
            out.push_str(&format!(
                "\n### Community {} Peers\n- {}\n",
                node.community,
                peers.join(", ")
            ));
        }

        Ok(out)
    }

    /// Shortest path between two concepts
    pub fn path(graph: &CodeGraph, start: &str, end: &str) -> Result<String, String> {
        let path_ids = graph
            .shortest_path(start, end)
            .ok_or_else(|| format!("No path found between '{}' and '{}'", start, end))?;

        let mut out = String::new();
        out.push_str(&format!("# Shortest Path: {} -> {}\n\n", start, end));
        out.push_str(&format!("Length: {} hops\n\n", path_ids.len().saturating_sub(1)));

        for (i, id) in path_ids.iter().enumerate() {
            let node = graph.nodes.iter().find(|n| &n.id == id);
            let label = node.map(|n| n.label.as_str()).unwrap_or(id.as_str());
            let loc = node.map(|n| format!(" ({}:{})", n.source_file, n.source_location)).unwrap_or_default();

            if i == 0 {
                out.push_str(&format!("{}. **[START]** `{}`{}\n", i + 1, label, loc));
            } else if i == path_ids.len() - 1 {
                out.push_str(&format!("{}. **[TARGET]** `{}`{}\n", i + 1, label, loc));
            } else {
                out.push_str(&format!("{}. `{}`{}\n", i + 1, label, loc));
            }
        }

        Ok(out)
    }

    /// BFS or DFS query traversal with LLM token budget limit
    pub fn query(
        graph: &CodeGraph,
        term: &str,
        opts: &QueryOptions,
    ) -> String {
        let term_lower = term.to_lowercase();
        // Match seeds
        let mut seeds: Vec<&crate::graph::GraphNode> = graph
            .nodes
            .iter()
            .filter(|n| n.label.to_lowercase().contains(&term_lower) || n.id.to_lowercase().contains(&term_lower))
            .collect();

        seeds.sort_by_key(|n| n.degree.unwrap_or(0));
        seeds.reverse();

        if seeds.is_empty() {
            return format!("No nodes matching '{}' found in graph.", term);
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut collected_nodes: Vec<&crate::graph::GraphNode> = Vec::new();
        let mut collected_edges: Vec<&crate::graph::GraphEdge> = Vec::new();

        if opts.dfs {
            // DFS traversal
            let mut stack: Vec<(&str, usize)> = Vec::new();
            for s in seeds.iter().take(5) {
                stack.push((&s.id, 0));
            }

            while let Some((curr_id, depth)) = stack.pop() {
                if depth > opts.max_depth || visited.contains(curr_id) {
                    continue;
                }
                visited.insert(curr_id.to_string());

                if let Some(node) = graph.nodes.iter().find(|n| n.id == curr_id) {
                    collected_nodes.push(node);
                }

                for edge in &graph.edges {
                    if edge.source == curr_id && !visited.contains(&edge.target) {
                        collected_edges.push(edge);
                        stack.push((&edge.target, depth + 1));
                    } else if !graph.directed && edge.target == curr_id && !visited.contains(&edge.source) {
                        collected_edges.push(edge);
                        stack.push((&edge.source, depth + 1));
                    }
                }
            }
        } else {
            // BFS traversal
            let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
            for s in seeds.iter().take(5) {
                queue.push_back((&s.id, 0));
                visited.insert(s.id.clone());
            }

            while let Some((curr_id, depth)) = queue.pop_front() {
                if let Some(node) = graph.nodes.iter().find(|n| n.id == curr_id) {
                    collected_nodes.push(node);
                }

                if depth >= opts.max_depth {
                    continue;
                }

                for edge in &graph.edges {
                    if edge.source == curr_id && !visited.contains(&edge.target) {
                        visited.insert(edge.target.clone());
                        collected_edges.push(edge);
                        queue.push_back((&edge.target, depth + 1));
                    } else if !graph.directed && edge.target == curr_id && !visited.contains(&edge.source) {
                        visited.insert(edge.source.clone());
                        collected_edges.push(edge);
                        queue.push_back((&edge.source, depth + 1));
                    }
                }
            }
        }

        // Format prompt context within token budget (~4 chars per token)
        let char_limit = opts.max_tokens * 4;
        let mut out = String::new();
        out.push_str(&format!("# Graph Query Context: \"{}\"\n", term));
        out.push_str(&format!(
            "Traversed: {} nodes, {} edges (Mode: {})\n\n",
            collected_nodes.len(),
            collected_edges.len(),
            if opts.dfs { "DFS" } else { "BFS" }
        ));

        out.push_str("## Relevant Symbols & Abstractions\n");
        for n in collected_nodes {
            let line = format!(
                "- `{}` [{}] ({}:{}) [{}]\n",
                n.label, n.file_type, n.source_file, n.source_location, n.provenance
            );
            if out.len() + line.len() >= char_limit {
                out.push_str("\n[... Context truncated by token budget ...]\n");
                return out;
            }
            out.push_str(&line);
        }

        out.push_str("\n## Structural Relationships\n");
        for e in collected_edges {
            let line = format!(
                "- `{}` --[{}]--> `{}` ({:?})\n",
                e.source, e.relation, e.target, e.provenance
            );
            if out.len() + line.len() >= char_limit {
                out.push_str("\n[... Context truncated by token budget ...]\n");
                return out;
            }
            out.push_str(&line);
        }

        out
    }

    /// Legacy symbol matcher for CLI compatibility.
    pub fn query_symbol(graph: &CodeGraph, term: &str) -> Vec<String> {
        let opts = QueryOptions::default();
        let res = Self::query(graph, term, &opts);
        vec![res]
    }
}
