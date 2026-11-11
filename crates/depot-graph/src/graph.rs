use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EdgeProvenance {
    Extracted,
    Inferred,
    Ambiguous,
}

impl Default for EdgeProvenance {
    fn default() -> Self {
        Self::Extracted
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default = "default_provenance")]
    pub provenance: String,
}

fn default_provenance() -> String {
    "EXTRACTED".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    #[serde(default)]
    pub provenance: EdgeProvenance,
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
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            // merge signature if missing
            if existing.signature.is_none() && node.signature.is_some() {
                existing.signature = node.signature;
            }
        } else {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let exists = self.edges.iter().any(|e| {
            e.source == edge.source && e.target == edge.target && e.relation == edge.relation
        });
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
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(limit);
        ranked
    }

    pub fn detect_communities(&mut self) -> usize {
        // Label propagation algorithm
        let mut node_to_comm: HashMap<String, usize> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            node_to_comm.insert(node.id.clone(), i);
        }

        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.source.clone()).or_default().push(edge.target.clone());
            adj.entry(edge.target.clone()).or_default().push(edge.source.clone());
        }

        for _ in 0..10 {
            let mut changed = false;
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
                    if let Some(curr) = node_to_comm.get_mut(&node.id) {
                        if *curr != best_comm {
                            *curr = best_comm;
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Remap to 0..K
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

    pub fn shortest_path(&self, start_term: &str, end_term: &str) -> Option<Vec<String>> {
        let start_node = self.find_node_fuzzy(start_term)?;
        let end_node = self.find_node_fuzzy(end_term)?;

        if start_node.id == end_node.id {
            return Some(vec![start_node.id.clone()]);
        }

        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
            if !self.directed {
                adj.entry(&edge.target).or_default().push(&edge.source);
            }
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut parent: HashMap<&str, &str> = HashMap::new();

        queue.push_back(&start_node.id);
        visited.insert(&start_node.id);

        let mut found = false;

        while let Some(current) = queue.pop_front() {
            if current == end_node.id {
                found = true;
                break;
            }

            if let Some(neighbors) = adj.get(current) {
                for &neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if !found {
            return None;
        }

        let mut path = Vec::new();
        let mut curr = end_node.id.as_str();
        path.push(curr.to_string());

        while let Some(&p) = parent.get(curr) {
            path.push(p.to_string());
            curr = p;
            if curr == start_node.id {
                break;
            }
        }

        path.reverse();
        Some(path)
    }

    pub fn find_node_fuzzy(&self, term: &str) -> Option<&GraphNode> {
        let term_lower = term.to_lowercase();
        // Exact label match
        if let Some(n) = self.nodes.iter().find(|n| n.label.eq_ignore_ascii_case(term)) {
            return Some(n);
        }
        // Exact id match
        if let Some(n) = self.nodes.iter().find(|n| n.id.eq_ignore_ascii_case(term)) {
            return Some(n);
        }
        // Contains label
        if let Some(n) = self.nodes.iter().find(|n| n.label.to_lowercase().contains(&term_lower)) {
            return Some(n);
        }
        // Contains id
        self.nodes.iter().find(|n| n.id.to_lowercase().contains(&term_lower))
    }
}
