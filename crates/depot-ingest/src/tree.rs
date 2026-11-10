use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct TreeNode {
    pub name: String,
    pub is_file: bool,
    pub lines: usize,
    pub tokens: usize,
    pub bytes: u64,
    pub children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    pub fn insert(&mut self, rel_path: &Path, lines: usize, tokens: usize, bytes: u64) {
        let mut current = self;
        let components: Vec<String> = rel_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        for (i, comp) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;
            let child = current
                .children
                .entry(comp.clone())
                .or_insert_with(|| TreeNode {
                    name: comp.clone(),
                    is_file: is_last,
                    lines: 0,
                    tokens: 0,
                    bytes: 0,
                    children: BTreeMap::new(),
                });

            if is_last {
                child.is_file = true;
                child.lines = lines;
                child.tokens = tokens;
                child.bytes = bytes;
            }
            current = child;
        }
    }

    pub fn render(&self, prefix: &str, is_last: bool) -> String {
        let mut out = String::new();
        let marker = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last { "    " } else { "│   " };

        if self.is_file {
            let size_str = format_bytes(self.bytes);
            let token_str = crate::tokenizer::Tokenizer::format_token_count(self.tokens);
            out.push_str(&format!(
                "{}{}{} ({} lines, {}, ~{} tokens)\n",
                prefix, marker, self.name, self.lines, size_str, token_str
            ));
        } else {
            out.push_str(&format!("{}{}{}/\n", prefix, marker, self.name));
            let total_children = self.children.len();
            for (idx, (_, child)) in self.children.iter().enumerate() {
                let child_is_last = idx == total_children - 1;
                let child_str = child.render(&format!("{}{}", prefix, new_prefix), child_is_last);
                out.push_str(&child_str);
            }
        }

        out
    }
}

pub fn build_tree_string(root_name: &str, files: &[(PathBuf, usize, usize, u64)]) -> String {
    let mut root = TreeNode {
        name: root_name.to_string(),
        is_file: false,
        lines: 0,
        tokens: 0,
        bytes: 0,
        children: BTreeMap::new(),
    };

    for (path, lines, tokens, bytes) in files {
        root.insert(path, *lines, *tokens, *bytes);
    }

    let mut out = format!("{}/\n", root_name);
    let total_children = root.children.len();
    for (idx, (_, child)) in root.children.iter().enumerate() {
        let is_last = idx == total_children - 1;
        out.push_str(&child.render("", is_last));
    }
    out
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
