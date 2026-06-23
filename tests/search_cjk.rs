mod common;

use common::{insert_record, TempDb};

#[test]
fn cjk_ngram_search_finds_two_character_terms() -> anyhow::Result<()> {
    let db = TempDb::new("search-cjk-short");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "支付失败排查规则",
        "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。",
        &["payment", "runbook"],
        Some("payment-service"),
    )?;

    let results = storage.search_records("支付", 10)?;

    assert!(results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.payment.failed")));

    Ok(())
}

#[test]
fn cjk_ngram_search_finds_multi_character_terms() -> anyhow::Result<()> {
    let db = TempDb::new("search-cjk-multi");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "支付失败排查规则",
        "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。",
        &["payment", "runbook"],
        Some("payment-service"),
    )?;

    let results = storage.search_records("支付失败", 10)?;

    assert!(results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.payment.failed")));

    Ok(())
}
