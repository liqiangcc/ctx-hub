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
        let actual = make_fts_query("payment");
        assert_eq!(actual, "\"payment\"");
    }

    #[test]
    fn preserves_dash_inside_quoted_token() {
        let actual = make_fts_query("payment-service");
        assert_eq!(actual, "\"payment-service\"");
    }

    #[test]
    fn splits_whitespace_into_required_terms() {
        let actual = make_fts_query("clean package");
        assert_eq!(actual, "\"clean\" \"package\"");
    }

    #[test]
    fn escapes_embedded_double_quotes() {
        let actual = make_fts_query("mock \"token\"");
        let expected = "\"mock\" \"\"\"token\"\"\"";
        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_query_stays_empty() {
        let actual = make_fts_query("   ");
        assert_eq!(actual, "");
    }
}
