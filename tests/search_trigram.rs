mod common;

use common::{insert_record, TempDb};

#[test]
fn trigram_search_finds_command_fragments_inside_tokens() -> anyhow::Result<()> {
    let db = TempDb::new("search-trigram");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "command.order.build",
        "Order service build",
        "mvn clean package -DskipTests -Ptest",
        &["build"],
        Some("order-service"),
    )?;

    let results = storage.search_records("skipTes", 10)?;
    let item = results
        .iter()
        .find(|item| item.key.as_deref() == Some("command.order.build"))
        .expect("expected command fragment search to find build command");

    assert_eq!(item.match_kind, "trigram");

    Ok(())
}

#[test]
fn trigram_search_finds_path_fragments() -> anyhow::Result<()> {
    let db = TempDb::new("search-trigram-path");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.order.api",
        "Order API runbook",
        "Check POST /internal/api/order/create when the order callback fails.",
        &["order", "runbook"],
        Some("order-service"),
    )?;

    let results = storage.search_records("api/order", 10)?;

    assert!(results
        .iter()
        .any(|item| item.key.as_deref() == Some("runbook.order.api")));

    Ok(())
}
