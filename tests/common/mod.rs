use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ctx_hub::core::record::RecordInput;
use ctx_hub::storage::sqlite::SqliteStorage;

pub struct TempDb {
    path: PathBuf,
}

impl TempDb {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ctx-hub-{prefix}-{}-{nanos}.db",
            std::process::id()
        ));
        Self { path }
    }

    pub fn open_storage(&self) -> anyhow::Result<SqliteStorage> {
        let storage = SqliteStorage::open(Some(&self.path))?;
        storage.init()?;
        Ok(storage)
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(wal_path(&self.path));
        let _ = std::fs::remove_file(shm_path(&self.path));
    }
}

pub fn insert_record(
    storage: &SqliteStorage,
    key: &str,
    title: &str,
    content: &str,
    tags: &[&str],
    service: Option<&str>,
) -> anyhow::Result<()> {
    let record = RecordInput::new(
        title.to_string(),
        content.to_string(),
        Some(key.to_string()),
        tags.iter().map(|tag| (*tag).to_string()).collect(),
        service.map(str::to_string),
        Some("test".to_string()),
        Some("search test".to_string()),
    )?;
    storage.insert_record(&record)
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}-wal", path.display()))
}

fn shm_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}-shm", path.display()))
}
