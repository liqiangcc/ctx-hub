pub fn make_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_single_token_in_quotes() {
        assert_eq!(make_fts_query("payment"), "\"payment\"");
    }

    #[test]
    fn preserves_dash_inside_quoted_token() {
        assert_eq!(make_fts_query("payment-service"), "\"payment-service\"");
    }

    #[test]
    fn splits_whitespace_into_required_terms() {
        assert_eq!(make_fts_query("clean package"), "\"clean\" \"package\"");
    }

    #[test]
    fn escapes_embedded_double_quotes() {
        assert_eq!(make_fts_query("mock \"token\""), "\"mock\" \"\"\"token\"\"\"");
    }

    #[test]
    fn empty_query_stays_empty() {
        assert_eq!(make_fts_query("   "), "");
    }
}
