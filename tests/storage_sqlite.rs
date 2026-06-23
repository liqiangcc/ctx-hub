use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ctx_hub::core::record::RecordInput;
use ctx_hub::storage::sqlite::SqliteStorage;

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("ctx-hub-storage-{}-{nanos}.db", std::process::id()))
}

#[test]
fn sqlite_storage_round_trip() -> anyhow::Result<()> {
    let db_path = temp_db_path();
    let storage = SqliteStorage::open(Some(&db_path))?;

    storage.init()?;
    assert_eq!(storage.record_count()?, 0);

    let record = RecordInput::new(
        "支付失败排查规则".to_string(),
        "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。".to_string(),
        Some("runbook.payment.failed".to_string()),
        vec!["payment".to_string(), "runbook".to_string()],
        Some("payment-service".to_string()),
        Some("test".to_string()),
        Some("storage test".to_string()),
    )?;

    storage.insert_record(&record)?;
    assert_eq!(storage.record_count()?, 1);

    let loaded = storage
        .get_record("runbook.payment.failed")?
        .expect("record should be found by key");
    assert_eq!(loaded.title, "支付失败排查规则");
    assert!(loaded.content.contains("payment_callback_log"));

    let zh_results = storage.search_records("支付", 10)?;
    assert!(zh_results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.payment.failed")));

    let service_results = storage.search_records("payment-service", 10)?;
    assert!(service_results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.payment.failed")));

    let tag_results = storage.search_by_tag("runbook", 10)?;
    assert!(tag_results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.payment.failed")));

    let tags = storage.list_tags()?;
    assert_eq!(tags.get("payment"), Some(&1));
    assert_eq!(tags.get("runbook"), Some(&1));

    storage.rebuild_index()?;

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));

    Ok(())
}
