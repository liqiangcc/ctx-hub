use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "ctx")]
#[command(about = "Context Hub SQLite FTS proof of concept")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
    Add {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Show {
        key_or_id: String,
    },
    Tag {
        tag: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    ListTags,
}

#[derive(Subcommand, Debug)]
enum DbCommand {
    Init,
    Info,
    RebuildIndex,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecordInput {
    id: String,
    key: Option<String>,
    title: String,
    content: String,
    tags: Vec<String>,
    service: Option<String>,
    env: Option<String>,
    source: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug)]
struct SearchResult {
    rowid: i64,
    key: Option<String>,
    title: String,
    tags_text: String,
    service: Option<String>,
    env: Option<String>,
    snippet: String,
    source: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = resolve_db_path(cli.db.as_ref())?;
    let conn = open_db(&db_path)?;

    match cli.command {
        Commands::Db { command } => match command {
            DbCommand::Init => {
                init_db(&conn)?;
                println!("initialized: {}", db_path.display());
            }
            DbCommand::Info => {
                init_db(&conn)?;
                let count: i64 =
                    conn.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
                println!("db: {}", db_path.display());
                println!("records: {count}");
            }
            DbCommand::RebuildIndex => {
                init_db(&conn)?;
                rebuild_index(&conn)?;
                println!("fts indexes rebuilt");
            }
        },
        Commands::Add {
            title,
            content,
            key,
            tags,
            service,
            env,
            source,
        } => {
            init_db(&conn)?;
            let record = RecordInput::new(title, content, key, tags, service, env, source)?;
            insert_record(&conn, &record)?;
            println!("added: {}", record.key.as_deref().unwrap_or(&record.id));
        }
        Commands::Search { query, limit } => {
            init_db(&conn)?;
            let results = search_records(&conn, &query, limit)?;
            print_results(&results);
        }
        Commands::Show { key_or_id } => {
            init_db(&conn)?;
            show_record(&conn, &key_or_id)?;
        }
        Commands::Tag { tag, limit } => {
            init_db(&conn)?;
            let results = search_by_tag(&conn, &tag, limit)?;
            print_results(&results);
        }
        Commands::ListTags => {
            init_db(&conn)?;
            list_tags(&conn)?;
        }
    }

    Ok(())
}

impl RecordInput {
    fn new(
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

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
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
        "#,
    )?;
    Ok(())
}

fn rebuild_index(conn: &Connection) -> Result<()> {
    conn.execute("INSERT INTO records_fts(records_fts) VALUES('rebuild')", [])?;
    conn.execute(
        "INSERT INTO records_trigram(records_trigram) VALUES('rebuild')",
        [],
    )?;
    Ok(())
}

fn insert_record(conn: &Connection, record: &RecordInput) -> Result<()> {
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

    conn.execute(
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

fn search_records(conn: &Connection, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    if let Some(exact) = search_exact_key(conn, query)? {
        seen.insert(exact.rowid);
        results.push(exact);
    }

    let fts_query = make_fts_query(query);
    if !fts_query.is_empty() {
        let mut stmt = conn.prepare(
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
                source: "fts".to_string(),
            })
        })?;
        for item in rows {
            let item = item?;
            if seen.insert(item.rowid) {
                results.push(item);
            }
        }
    }

    if query.chars().count() >= 3 && results.len() < limit {
        let mut stmt = conn.prepare(
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
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(SearchResult {
                rowid: row.get(0)?,
                key: row.get(1)?,
                title: row.get(2)?,
                tags_text: row.get(3)?,
                service: row.get(4)?,
                env: row.get(5)?,
                snippet: row.get(6)?,
                source: "trigram".to_string(),
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

fn search_exact_key(conn: &Connection, query: &str) -> Result<Option<SearchResult>> {
    let mut stmt = conn.prepare(
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
            source: "exact".to_string(),
        }))
    } else {
        Ok(None)
    }
}

fn search_by_tag(conn: &Connection, tag: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let pattern = format!("%{}%", tag);
    let mut stmt = conn.prepare(
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
            source: "tag".to_string(),
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn show_record(conn: &Connection, key_or_id: &str) -> Result<()> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, key, title, content, tags_text, service, env, source, created_at, updated_at
        FROM records
        WHERE key = ?1 OR id = ?1
        LIMIT 1
        "#,
    )?;
    let mut rows = stmt.query(params![key_or_id])?;
    if let Some(row) = rows.next()? {
        println!("id: {}", row.get::<_, String>(0)?);
        println!(
            "key: {}",
            row.get::<_, Option<String>>(1)?.unwrap_or_default()
        );
        println!("title: {}", row.get::<_, String>(2)?);
        println!("tags: {}", row.get::<_, String>(4)?);
        println!(
            "service: {}",
            row.get::<_, Option<String>>(5)?.unwrap_or_default()
        );
        println!(
            "env: {}",
            row.get::<_, Option<String>>(6)?.unwrap_or_default()
        );
        println!(
            "source: {}",
            row.get::<_, Option<String>>(7)?.unwrap_or_default()
        );
        println!("created_at: {}", row.get::<_, String>(8)?);
        println!("updated_at: {}", row.get::<_, String>(9)?);
        println!("\n{}", row.get::<_, String>(3)?);
    } else {
        println!("not found: {key_or_id}");
    }
    Ok(())
}

fn list_tags(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT tags_text FROM records WHERE tags_text <> ''")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        for tag in row?.split_whitespace() {
            *counts.entry(tag.to_string()).or_insert(0usize) += 1;
        }
    }
    for (tag, count) in counts {
        println!("{tag}\t{count}");
    }
    Ok(())
}

fn print_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("no results");
        return;
    }
    for (idx, item) in results.iter().enumerate() {
        println!(
            "[{}] {}",
            idx + 1,
            item.key.as_deref().unwrap_or("<no-key>")
        );
        println!("title: {}", item.title);
        println!("tags: {}", item.tags_text);
        if let Some(service) = &item.service {
            println!("service: {service}");
        }
        if let Some(env) = &item.env {
            println!("env: {env}");
        }
        println!("source: {}", item.source);
        println!("snippet: {}", item.snippet.replace('\n', " "));
        println!();
    }
}

fn make_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn make_search_ngrams(text: &str) -> String {
    let mut grams = Vec::new();
    let mut current = Vec::new();

    for ch in text.chars() {
        if is_cjk(ch) {
            current.push(ch);
        } else {
            push_ngrams(&current, &mut grams);
            current.clear();
        }
    }
    push_ngrams(&current, &mut grams);

    grams.sort();
    grams.dedup();
    grams.into_iter().collect::<Vec<_>>().join(" ")
}

fn push_ngrams(chars: &[char], grams: &mut Vec<String>) {
    for n in 2..=3 {
        if chars.len() < n {
            continue;
        }
        for window in chars.windows(n) {
            grams.push(window.iter().collect());
        }
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
    )
}
