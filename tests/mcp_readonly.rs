mod common;

use common::{insert_record, TempDb};
use ctx_hub::mcp::{
    ReadOnlyMcp, GET_CONTEXT_BY_KEY, GET_SERVICE_CONTEXT, LIST_TAGS, SEARCH_CONTEXT,
};
use serde_json::json;

#[test]
fn mcp_exposes_only_read_only_tools() -> anyhow::Result<()> {
    let db = TempDb::new("mcp-tools");
    let storage = db.open_storage()?;
    let mcp = ReadOnlyMcp::new(&storage);

    let names = mcp
        .tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            SEARCH_CONTEXT,
            GET_CONTEXT_BY_KEY,
            LIST_TAGS,
            GET_SERVICE_CONTEXT
        ]
    );
    assert!(!names.iter().any(|name| {
        matches!(
            *name,
            "add_context" | "update_context" | "delete_context" | "import_context"
        )
    }));

    Ok(())
}

#[test]
fn mcp_tools_read_context_without_mutating_storage() -> anyhow::Result<()> {
    let db = TempDb::new("mcp-readonly");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "支付失败排查规则",
        "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。",
        &["payment", "runbook"],
        Some("payment-service"),
    )?;
    insert_record(
        &storage,
        "command.order.build",
        "order-service 构建命令",
        "mvn clean package -DskipTests -Ptest",
        &["build"],
        Some("order-service"),
    )?;

    let count_before = storage.record_count()?;
    let mcp = ReadOnlyMcp::new(&storage);

    let search = mcp.call_tool(SEARCH_CONTEXT, json!({"query": "payment_callback_log"}))?;
    assert_eq!(
        search["results"][0]["key"].as_str(),
        Some("runbook.payment.failed")
    );

    let detail = mcp.call_tool(GET_CONTEXT_BY_KEY, json!({"key": "runbook.payment.failed"}))?;
    assert_eq!(detail["record"]["title"].as_str(), Some("支付失败排查规则"));
    assert_eq!(
        detail["record"]["content"].as_str(),
        Some("支付失败时先查询 payment_callback_log，再查询 payment-service 日志。")
    );

    let tags = mcp.call_tool(LIST_TAGS, json!({}))?;
    assert_eq!(tags["tags"]["payment"].as_u64(), Some(1));
    assert_eq!(tags["tags"]["build"].as_u64(), Some(1));

    let service = mcp.call_tool(
        GET_SERVICE_CONTEXT,
        json!({"service": "payment-service", "limit": 10}),
    )?;
    let service_results = service["results"].as_array().expect("results array");
    assert_eq!(service_results.len(), 1);
    assert_eq!(
        service_results[0]["key"].as_str(),
        Some("runbook.payment.failed")
    );
    assert_eq!(
        service_results[0]["service"].as_str(),
        Some("payment-service")
    );

    assert_eq!(storage.record_count()?, count_before);
    Ok(())
}

#[test]
fn mcp_json_rpc_lists_and_calls_tools() -> anyhow::Result<()> {
    let db = TempDb::new("mcp-json-rpc");
    let storage = db.open_storage()?;

    insert_record(
        &storage,
        "runbook.payment.failed",
        "Payment failure runbook",
        "Check payment_callback_log before retrying.",
        &["payment", "runbook"],
        Some("payment-service"),
    )?;

    let mcp = ReadOnlyMcp::new(&storage);
    let tools = mcp
        .handle_json_rpc(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))?
        .expect("tools/list response");

    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 4);

    let call = mcp
        .handle_json_rpc(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": SEARCH_CONTEXT,
                "arguments": {"query": "payment_callback_log"}
            }
        }))?
        .expect("tools/call response");

    assert_eq!(
        call["result"]["structuredContent"]["results"][0]["key"].as_str(),
        Some("runbook.payment.failed")
    );
    assert!(call["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("payment_callback_log"));

    Ok(())
}
