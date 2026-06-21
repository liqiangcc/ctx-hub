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

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    fn migration_count(conn: &Connection) -> Result<i64> {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM schema_migrations")?;
        let count = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    #[test]
    fn creates_schema_migrations_table() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        ensure_schema_version(&conn)?;

        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
        )?;
        let table_name: String = stmt.query_row([], |row| row.get(0))?;

        assert_eq!(table_name, "schema_migrations");
        Ok(())
    }

    #[test]
    fn records_current_schema_version() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        ensure_schema_version(&conn)?;

        let version = current_schema_version(&conn)?;
        assert_eq!(version, Some(CURRENT_SCHEMA_VERSION));
        Ok(())
    }

    #[test]
    fn ensure_schema_version_is_idempotent() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        ensure_schema_version(&conn)?;
        ensure_schema_version(&conn)?;

        assert_eq!(migration_count(&conn)?, 1);
        Ok(())
    }
}
