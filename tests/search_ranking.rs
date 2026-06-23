mod common;

use common::{insert_record, TempDb};

#[test]
fn exact_key_match_is_returned_first() -> anyhow::Result<()> {
    let db = TempDb::new("search-ranking");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "Payment failure runbook",
        "Sparse record with an exact key.",
        &["payment"],
        Some("payment-service"),
    )?;
    insert_record(
        &storage,
        "notes.payment.failed",
        "Payment failed analysis",
        "runbook.payment.failed appears many times in this full text body.",
        &["payment", "analysis"],
        Some("payment-service"),
    )?;

    let results = storage.search_records("runbook.payment.failed", 10)?;

    assert_eq!(
        results.first().and_then(|item| item.key.as_deref()),
        Some("runbook.payment.failed")
    );
    assert_eq!(
        results.first().map(|item| item.match_kind.as_str()),
        Some("exact")
    );

    Ok(())
}

#[test]
fn exact_key_match_respects_limit_one() -> anyhow::Result<()> {
    let db = TempDb::new("search-ranking-limit");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "command.order.build",
        "Order build command",
        "mvn clean package -DskipTests",
        &["build"],
        Some("order-service"),
    )?;
    insert_record(
        &storage,
        "notes.order.build",
        "Order build note",
        "command.order.build is referenced by this note.",
        &["build"],
        Some("order-service"),
    )?;

    let results = storage.search_records("command.order.build", 1)?;

    assert_eq!(results.len(), 1);
    assert_eq!(
        results.first().and_then(|item| item.key.as_deref()),
        Some("command.order.build")
    );

    Ok(())
}
