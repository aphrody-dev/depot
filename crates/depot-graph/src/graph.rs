use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub file_type: String,
    pub source_file: String,
    pub source_location: String,
    pub community: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degree: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub directed: bool,
    pub multigraph: bool,
    pub graph: HashMap<String, String>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self {
            directed: false,
            multigraph: false,
            graph: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: GraphNode) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let exists = self
            .edges
            .iter()
            .any(|e| e.source == edge.source && e.target == edge.target && e.relation == edge.relation);
        if !exists {
            self.edges.push(edge);
        }
    }

    pub fn compute_degrees_and_god_nodes(&mut self, limit: usize) -> Vec<(String, usize)> {
        let mut degrees: HashMap<String, usize> = HashMap::new();

        for edge in &self.edges {
            *degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        for node in &mut self.nodes {
            let deg = *degrees.get(&node.id).unwrap_or(&0);
            node.degree = Some(deg);
        }

        let mut ranked: Vec<(String, usize)> = degrees.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.truncate(limit);
        ranked
    }

    pub fn detect_communities(&mut self) -> usize {
        // Label propagation algorithm for fast deterministic community detection
        let mut node_to_comm: HashMap<String, usize> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            node_to_comm.insert(node.id.clone(), i);
        }

        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.source.clone()).or_default().push(edge.target.clone());
            adj.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }

        for _ in 0..5 {
            for node in &self.nodes {
                let neighbors = adj.get(&node.id).cloned().unwrap_or_default();
                if neighbors.is_empty() {
                    continue;
                }

                let mut counts: HashMap<usize, usize> = HashMap::new();
                for neigh in neighbors {
                    if let Some(&c) = node_to_comm.get(&neigh) {
                        *counts.entry(c).or_insert(0) += 1;
                    }
                }

                if let Some((&best_comm, _)) = counts.iter().max_by_key(|entry| entry.1) {
                    node_to_comm.insert(node.id.clone(), best_comm);
                }
            }
        }

        // Remap communities to 0..K
        let mut comm_map: HashMap<usize, usize> = HashMap::new();
        let mut next_id = 0;

        for node in &mut self.nodes {
            let raw = *node_to_comm.get(&node.id).unwrap_or(&0);
            let mapped = *comm_map.entry(raw).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            node.community = mapped;
        }

        next_id
    }
}
