use serde::{Deserialize, Serialize};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::languages::SupportedLanguage;
use crate::symbols::Span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCaptureItem {
    pub pattern_index: usize,
    pub capture_name: String,
    pub text: String,
    pub span: Span,
}

pub struct QueryRunner;

impl QueryRunner {
    pub fn execute(
        lang: SupportedLanguage,
        source: &str,
        query_str: &str,
    ) -> Result<Vec<QueryCaptureItem>, String> {
        let ts_lang = lang.tree_sitter_language();
        let query = Query::new(&ts_lang, query_str)
            .map_err(|e| format!("Query compile error: {}", e))?;

        let mut parser = Parser::new();
        parser
            .set_language(&ts_lang)
            .map_err(|e| format!("Failed to set language on parser: {:?}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Tree-sitter failed to parse source".to_string())?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let mut results = Vec::new();
        let capture_names = query.capture_names();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let name = capture_names
                    .get(capture.index as usize)
                    .copied()
                    .unwrap_or("unknown")
                    .to_string();
                let span = Span::from_node(&capture.node);
                let text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string();

                results.push(QueryCaptureItem {
                    pattern_index: m.pattern_index,
                    capture_name: name,
                    text,
                    span,
                });
            }
        }

        Ok(results)
    }
}
