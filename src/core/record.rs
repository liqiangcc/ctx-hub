use anyhow::Result;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordInput {
    pub id: String,
    pub key: Option<String>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub service: Option<String>,
    pub env: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct RecordDetail {
    pub id: String,
    pub key: Option<String>,
    pub title: String,
    pub content: String,
    pub tags_text: String,
    pub service: Option<String>,
    pub env: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl RecordInput {
    pub fn new(
        title: String,
        content: String,
        key: Option<String>,
        tags: Vec<String>,
        service: Option<String>,
        env: Option<String>,
        source: Option<String>,
    ) -> Result<Self> {
        let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
        Ok(Self {
            id: format!("ctx_{}", Uuid::new_v4().simple()),
            key,
            title,
            content,
            tags,
            service,
            env,
            source,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}
