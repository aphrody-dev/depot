// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Helix Editor transactional Rope editing engine (`helix-core` pattern).
//! Uses `ropey::Rope` for zero-copy, byte-accurate, transactional span replacement.
//! Non-overlapping `(start_byte, end_byte, replacement)` edits are applied in reverse byte order
//! so all Tree-Sitter AST byte offsets remain valid in a single O(k log N) pass.

use ropey::Rope;

/// Applies non-overlapping `(start_byte, end_byte, replacement)` edits in reverse byte order.
/// Tree-sitter byte offsets remain valid without invalidation cascades.
pub fn apply_helix_rope_edits(source: &str, mut edits: Vec<(usize, usize, String)>) -> String {
    if edits.is_empty() {
        return source.to_string();
    }
    // Sort in reverse byte order so subsequent replacements don't shift earlier byte offsets
    edits.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut rope = Rope::from_str(source);
    let mut last_start = usize::MAX;

    for (start_byte, end_byte, replacement) in edits {
        if end_byte > last_start || start_byte > end_byte || end_byte > rope.len_bytes() {
            continue;
        }
        let start_char = rope.byte_to_char(start_byte);
        let end_char = rope.byte_to_char(end_byte);
        rope.remove(start_char..end_char);
        rope.insert(start_char, &replacement);
        last_start = start_byte;
    }
    rope.to_string()
}

/// Transactional Rope Editing Session allowing staged modifications before committing.
pub struct RopeSession {
    rope: Rope,
    staged_edits: Vec<(usize, usize, String)>,
}

impl RopeSession {
    pub fn new(source: &str) -> Self {
        Self {
            rope: Rope::from_str(source),
            staged_edits: Vec::new(),
        }
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Stage a byte-span replacement `[start_byte, end_byte) -> replacement`.
    pub fn stage_replace(&mut self, start_byte: usize, end_byte: usize, replacement: impl Into<String>) {
        self.staged_edits.push((start_byte, end_byte, replacement.into()));
    }

    /// Stage an insertion at `byte_offset`.
    pub fn stage_insert(&mut self, byte_offset: usize, text: impl Into<String>) {
        self.staged_edits.push((byte_offset, byte_offset, text.into()));
    }

    /// Stage a deletion of `[start_byte, end_byte)`.
    pub fn stage_delete(&mut self, start_byte: usize, end_byte: usize) {
        self.staged_edits.push((start_byte, end_byte, String::new()));
    }

    /// Commits all staged edits in reverse byte order and returns the resulting string.
    pub fn commit(mut self) -> String {
        let edits = std::mem::take(&mut self.staged_edits);
        let current_text = self.rope.to_string();
        apply_helix_rope_edits(&current_text, edits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_helix_rope_edits() {
        let src = "fn old_func() { let x = 10; }";
        let edits = vec![
            (3, 11, "new_func".to_string()),
            (24, 26, "42".to_string()),
        ];
        let result = apply_helix_rope_edits(src, edits);
        assert_eq!(result, "fn new_func() { let x = 42; }");
    }

    #[test]
    fn test_rope_session() {
        let mut session = RopeSession::new("const a = 1;");
        session.stage_replace(0, 5, "let");
        session.stage_replace(10, 11, "42");
        let result = session.commit();
        assert_eq!(result, "let a = 42;");
    }
}
