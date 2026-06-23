mod common;

use common::{insert_record, TempDb};

#[test]
fn fts_searches_content_service_and_tags() -> anyhow::Result<()> {
    let db = TempDb::new("search-fts");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "Payment failure runbook",
        "Check payment_callback_log before retrying the callback worker.",
        &["payment", "runbook"],
        Some("payment-service"),
    )?;

    for query in ["payment", "payment-service", "runbook"] {
        let results = storage.search_records(query, 10)?;
        assert!(
            results
                .iter()
                .any(|item| item.key.as_deref() == Some("runbook.payment.failed")),
            "expected query {query:?} to find payment runbook"
        );
    }

    let service_results = storage.search_records("payment-service", 10)?;
    let service_match = service_results
        .iter()
        .find(|item| item.key.as_deref() == Some("runbook.payment.failed"))
        .expect("expected service-name search to find payment runbook");
    assert!(service_match.snippet.contains("[payment-service]"));

    Ok(())
}

#[test]
fn tag_command_search_uses_exact_tag_boundaries() -> anyhow::Result<()> {
    let db = TempDb::new("search-tags");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "Payment runbook",
        "Payment runbook content.",
        &["runbook"],
        Some("payment-service"),
    )?;
    insert_record(
        &storage,
        "notes.story",
        "Story note",
        "A note with a longer tag.",
        &["storybook"],
        Some("docs-service"),
    )?;

    let run_results = storage.search_by_tag("run", 10)?;
    assert!(run_results.is_empty());

    let runbook_results = storage.search_by_tag("runbook", 10)?;
    assert_eq!(runbook_results.len(), 1);
    assert_eq!(
        runbook_results.first().and_then(|item| item.key.as_deref()),
        Some("runbook.payment.failed")
    );

    Ok(())
}

#[test]
fn special_query_characters_do_not_break_search() -> anyhow::Result<()> {
    let db = TempDb::new("search-escaping");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.special.query",
        "Special query runbook",
        "Investigate mock \"token\", status:500, payment-service, and /api/order errors.",
        &["runbook"],
        Some("payment-service"),
    )?;

    for query in [
        "mock \"token\"",
        "status:500",
        "payment-service",
        "/api/order",
        "(retry)",
    ] {
        let _ = storage.search_records(query, 10)?;
    }

    Ok(())
}

#[test]
fn snippets_highlight_matching_content() -> anyhow::Result<()> {
    let db = TempDb::new("search-snippet");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.retry.callback",
        "Callback retry runbook",
        "When callback delivery fails, retry the callback worker after checking logs.",
        &["runbook"],
        Some("payment-service"),
    )?;

    let results = storage.search_records("retry", 10)?;
    let item = results
        .iter()
        .find(|item| item.key.as_deref() == Some("runbook.retry.callback"))
        .expect("expected retry runbook");

    assert!(item.snippet.contains("[retry]"));

    Ok(())
}
