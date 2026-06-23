use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

use crate::core::ngram::make_search_ngrams;
use crate::core::query::make_fts_query;
use crate::core::record::{RecordDetail, RecordInput};
use crate::core::search::SearchResult;
use crate::storage::{schema, Storage};

pub struct SqliteStorage {
    conn: Connection,
    db_path: PathBuf,
}

impl SqliteStorage {
    pub fn open(cli_db: Option<&PathBuf>) -> Result<Self> {
        let db_path = resolve_db_path(cli_db)?;
        let conn = open_db(&db_path)?;
        Ok(Self { conn, db_path })
    }

    pub fn path(&self) -> &Path {
        &self.db_path
    }

    pub fn init(&self) -> Result<()> {
        schema::init_schema(&self.conn)
    }

    pub fn record_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))
            .map_err(Into::into)
    }

    pub fn rebuild_index(&self) -> Result<()> {
        self.conn
            .execute("INSERT INTO records_fts(records_fts) VALUES('rebuild')", [])?;
        self.conn.execute(
            "INSERT INTO records_trigram(records_trigram) VALUES('rebuild')",
            [],
        )?;
        Ok(())
    }

    pub fn insert_record(&self, record: &RecordInput) -> Result<()> {
        let tags_json = serde_json::to_string(&record.tags)?;
        let tags_text = record.tags.join(" ");
        let search_ngrams = make_search_ngrams(&format!(
            "{} {} {} {} {} {}",
            record.title,
            record.content,
            tags_text,
            record.service.as_deref().unwrap_or_default(),
            record.env.as_deref().unwrap_or_default(),
            record.source.as_deref().unwrap_or_default()
        ));

        self.conn.execute(
            r#"
            INSERT INTO records (
              id, key, title, content, tags_json, tags_text,
              service, env, source, created_at, updated_at, search_ngrams
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                record.id,
                record.key,
                record.title,
                record.content,
                tags_json,
                tags_text,
                record.service,
                record.env,
                record.source,
                record.created_at,
                record.updated_at,
                search_ngrams,
            ],
        )?;
        Ok(())
    }

    pub fn search_records(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        if let Some(exact) = self.search_exact_key(query)? {
            seen.insert(exact.rowid);
            results.push(exact);
        }

        let fts_query = make_fts_query(query);
        if !fts_query.is_empty() {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT
                  r.rowid,
                  r.key,
                  r.title,
                  r.tags_text,
                  r.service,
                  r.env,
                  snippet(records_fts, 2, '[', ']', '...', 32) AS snippet,
                  bm25(records_fts, 8.0, 10.0, 3.0, 5.0, 4.0, 2.0, 2.0, 1.0) AS rank
                FROM records_fts
                JOIN records r ON r.rowid = records_fts.rowid
                WHERE records_fts MATCH ?1
                ORDER BY rank ASC, r.usage_count DESC, r.updated_at DESC
                LIMIT ?2
                "#,
            )?;
            let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
                Ok(SearchResult {
                    rowid: row.get(0)?,
                    key: row.get(1)?,
                    title: row.get(2)?,
                    tags_text: row.get(3)?,
                    service: row.get(4)?,
                    env: row.get(5)?,
                    snippet: row.get(6)?,
                    match_kind: "fts".to_string(),
                })
            })?;
            for item in rows {
                let item = item?;
                if seen.insert(item.rowid) {
                    results.push(item);
                }
            }
        }

        let trigram_query = make_fts_query(query);
        if !trigram_query.is_empty() && query.chars().count() >= 3 && results.len() < limit {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT
                  r.rowid,
                  r.key,
                  r.title,
                  r.tags_text,
                  r.service,
                  r.env,
                  substr(r.content, 1, 160) AS snippet
                FROM records_trigram
                JOIN records r ON r.rowid = records_trigram.rowid
                WHERE records_trigram MATCH ?1
                LIMIT ?2
                "#,
            )?;
            let rows = stmt.query_map(params![trigram_query, limit as i64], |row| {
                Ok(SearchResult {
                    rowid: row.get(0)?,
                    key: row.get(1)?,
                    title: row.get(2)?,
                    tags_text: row.get(3)?,
                    service: row.get(4)?,
                    env: row.get(5)?,
                    snippet: row.get(6)?,
                    match_kind: "trigram".to_string(),
                })
            })?;
            for item in rows {
                let item = item?;
                if seen.insert(item.rowid) {
                    results.push(item);
                }
            }
        }

        results.truncate(limit);
        Ok(results)
    }

    pub fn get_record(&self, key_or_id: &str) -> Result<Option<RecordDetail>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, key, title, content, tags_text, service, env, source, created_at, updated_at
            FROM records
            WHERE key = ?1 OR id = ?1
            LIMIT 1
            "#,
        )?;
        let mut rows = stmt.query(params![key_or_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(RecordDetail {
                id: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                tags_text: row.get(4)?,
                service: row.get(5)?,
                env: row.get(6)?,
                source: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn search_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let pattern = format!("%{}%", tag);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT rowid, key, title, tags_text, service, env, substr(content, 1, 160)
            FROM records
            WHERE tags_text LIKE ?1
            ORDER BY updated_at DESC
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(SearchResult {
                rowid: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                tags_text: row.get(3)?,
                service: row.get(4)?,
                env: row.get(5)?,
                snippet: row.get(6)?,
                match_kind: "tag".to_string(),
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn list_tags(&self) -> Result<BTreeMap<String, usize>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tags_text FROM records WHERE tags_text <> ''")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut counts = BTreeMap::new();
        for row in rows {
            for tag in row?.split_whitespace() {
                *counts.entry(tag.to_string()).or_insert(0usize) += 1;
            }
        }
        Ok(counts)
    }

    fn search_exact_key(&self, query: &str) -> Result<Option<SearchResult>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT rowid, key, title, tags_text, service, env, substr(content, 1, 160)
            FROM records
            WHERE key = ?1 OR id = ?1
            LIMIT 1
            "#,
        )?;
        let mut rows = stmt.query(params![query])?;
        if let Some(row) = rows.next()? {
            Ok(Some(SearchResult {
                rowid: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                tags_text: row.get(3)?,
                service: row.get(4)?,
                env: row.get(5)?,
                snippet: row.get(6)?,
                match_kind: "exact".to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}

impl Storage for SqliteStorage {
    fn path(&self) -> &Path {
        SqliteStorage::path(self)
    }

    fn init(&self) -> Result<()> {
        SqliteStorage::init(self)
    }

    fn record_count(&self) -> Result<i64> {
        SqliteStorage::record_count(self)
    }

    fn rebuild_index(&self) -> Result<()> {
        SqliteStorage::rebuild_index(self)
    }

    fn insert_record(&self, record: &RecordInput) -> Result<()> {
        SqliteStorage::insert_record(self, record)
    }

    fn search_records(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        SqliteStorage::search_records(self, query, limit)
    }

    fn get_record(&self, key_or_id: &str) -> Result<Option<RecordDetail>> {
        SqliteStorage::get_record(self, key_or_id)
    }

    fn search_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<SearchResult>> {
        SqliteStorage::search_by_tag(self, tag, limit)
    }

    fn list_tags(&self) -> Result<BTreeMap<String, usize>> {
        SqliteStorage::list_tags(self)
    }
}

fn resolve_db_path(cli_db: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_db {
        return Ok(path.clone());
    }
    if let Ok(path) = env::var("CTX_HUB_DB") {
        return Ok(PathBuf::from(path));
    }
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .context("cannot find HOME or USERPROFILE for default db path")?;
    Ok(PathBuf::from(home).join(".ctx-hub").join("ctx-hub.db"))
}

fn open_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create db directory: {}", parent.display()))?;
    }
    Connection::open(path).with_context(|| format!("failed to open db: {}", path.display()))
}
