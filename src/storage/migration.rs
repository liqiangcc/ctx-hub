use anyhow::Result;
use rusqlite::{params, Connection};

pub const CURRENT_SCHEMA_VERSION: i64 = 1;

pub fn ensure_schema_version(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO schema_migrations(version) VALUES (?1)",
        params![CURRENT_SCHEMA_VERSION],
    )?;

    Ok(())
}

pub fn current_schema_version(conn: &Connection) -> Result<Option<i64>> {
    let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations")?;
    let version = stmt.query_row([], |row| row.get::<_, Option<i64>>(0))?;
    Ok(version)
}
