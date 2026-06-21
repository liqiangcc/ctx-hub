use anyhow::Result;
use rusqlite::Connection;

use crate::storage::migration;

const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS records (
  rowid INTEGER PRIMARY KEY AUTOINCREMENT,
  id TEXT NOT NULL UNIQUE,
  key TEXT UNIQUE,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  tags_json TEXT NOT NULL DEFAULT '[]',
  tags_text TEXT NOT NULL DEFAULT '',
  service TEXT,
  env TEXT,
  source TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  usage_count INTEGER NOT NULL DEFAULT 0,
  search_ngrams TEXT NOT NULL DEFAULT ''
);

CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
  key,
  title,
  content,
  tags_text,
  service,
  env,
  source,
  search_ngrams,
  content='records',
  content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 0',
  prefix='2 3 4'
);

CREATE VIRTUAL TABLE IF NOT EXISTS records_trigram USING fts5(
  title,
  content,
  key,
  tags_text,
  service,
  env,
  source,
  content='records',
  content_rowid='rowid',
  tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
  INSERT INTO records_fts(rowid, key, title, content, tags_text, service, env, source, search_ngrams)
  VALUES (new.rowid, new.key, new.title, new.content, new.tags_text, new.service, new.env, new.source, new.search_ngrams);

  INSERT INTO records_trigram(rowid, title, content, key, tags_text, service, env, source)
  VALUES (new.rowid, new.title, new.content, new.key, new.tags_text, new.service, new.env, new.source);
END;
"#;

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    migration::ensure_schema_version(conn)?;
    Ok(())
}
