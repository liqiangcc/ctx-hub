use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use crate::core::record::{RecordDetail, RecordInput};
use crate::core::search::SearchResult;

pub mod migration;
pub mod schema;
pub mod sqlite;

pub trait Storage {
    fn path(&self) -> &Path;
    fn init(&self) -> Result<()>;
    fn record_count(&self) -> Result<i64>;
    fn rebuild_index(&self) -> Result<()>;
    fn insert_record(&self, record: &RecordInput) -> Result<()>;
    fn search_records(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    fn get_record(&self, key_or_id: &str) -> Result<Option<RecordDetail>>;
    fn search_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<SearchResult>>;
    fn search_by_service(&self, service: &str, limit: usize) -> Result<Vec<SearchResult>>;
    fn list_tags(&self) -> Result<BTreeMap<String, usize>>;
}
