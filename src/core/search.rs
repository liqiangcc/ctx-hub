#[derive(Debug)]
pub struct SearchResult {
    pub rowid: i64,
    pub key: Option<String>,
    pub title: String,
    pub tags_text: String,
    pub service: Option<String>,
    pub env: Option<String>,
    pub snippet: String,
    pub match_kind: String,
}
