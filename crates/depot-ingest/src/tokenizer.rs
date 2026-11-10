pub struct Tokenizer;

impl Tokenizer {
    /// Fast and accurate token estimation for LLM contexts (aligned with OpenAI/Gemini BPE).
    /// Roughly ~3.7 characters per token on code, accurately handling words, whitespace, and symbols.
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let mut tokens = 0;
        let mut in_word = false;

        for c in text.chars() {
            if c.is_alphanumeric() || c == '_' {
                if !in_word {
                    tokens += 1;
                    in_word = true;
                }
            } else if c.is_whitespace() {
                in_word = false;
                if c == '\n' {
                    tokens += 1;
                }
            } else {
                // Punctuation and symbols are usually 1 token each
                tokens += 1;
                in_word = false;
            }
        }

        // Clamp to a reasonable character-ratio bound
        let char_based = (text.len() as f64 / 3.8).ceil() as usize;
        (tokens + char_based) / 2
    }

    pub fn format_token_count(tokens: usize) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}k", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }
}
