use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

use crate::core::record::RecordInput;

pub const JSONL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonlRecord {
    pub schema_version: u32,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImportSummary {
    pub imported: usize,
    pub skipped_duplicates: usize,
}

impl JsonlRecord {
    pub fn into_record_input(self) -> Result<RecordInput> {
        if self.schema_version != JSONL_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported JSONL schema version {}; expected {}",
                self.schema_version,
                JSONL_SCHEMA_VERSION
            );
        }

        Ok(RecordInput {
            id: self.id,
            key: self.key,
            title: self.title,
            content: self.content,
            tags: self.tags,
            service: self.service,
            env: self.env,
            source: self.source,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

pub fn write_jsonl<W: Write>(records: &[JsonlRecord], mut writer: W) -> Result<()> {
    for record in records {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

pub fn read_jsonl<R: BufRead>(reader: R) -> Result<Vec<JsonlRecord>> {
    let mut records = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<JsonlRecord>(&line)
            .with_context(|| format!("invalid JSONL record on line {}", idx + 1))?;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> JsonlRecord {
        JsonlRecord {
            schema_version: JSONL_SCHEMA_VERSION,
            id: "ctx_test".to_string(),
            key: Some("runbook.payment.failed".to_string()),
            title: "Payment failure runbook".to_string(),
            content: "Check payment_callback_log.".to_string(),
            tags: vec!["payment".to_string(), "runbook".to_string()],
            service: Some("payment-service".to_string()),
            env: Some("test".to_string()),
            source: Some("unit test".to_string()),
            created_at: "2026-06-23T00:00:00Z".to_string(),
            updated_at: "2026-06-23T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn writes_and_reads_jsonl_records() -> Result<()> {
        let records = vec![sample_record()];
        let mut output = Vec::new();

        write_jsonl(&records, &mut output)?;
        let loaded = read_jsonl(output.as_slice())?;

        assert_eq!(loaded, records);
        Ok(())
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut record = sample_record();
        record.schema_version = JSONL_SCHEMA_VERSION + 1;

        let err = record
            .into_record_input()
            .expect_err("unsupported schema should fail");
        assert!(err.to_string().contains("unsupported JSONL schema version"));
    }
}
