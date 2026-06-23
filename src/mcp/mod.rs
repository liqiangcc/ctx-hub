use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use crate::core::record::RecordDetail;
use crate::core::search::SearchResult;
use crate::storage::Storage;

pub const SEARCH_CONTEXT: &str = "search_context";
pub const GET_CONTEXT_BY_KEY: &str = "get_context_by_key";
pub const LIST_TAGS: &str = "list_tags";
pub const GET_SERVICE_CONTEXT: &str = "get_service_context";

const DEFAULT_LIMIT: usize = 10;

pub struct ReadOnlyMcp<'a, S: Storage> {
    storage: &'a S,
}

#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct KeyArgs {
    key: String,
}

#[derive(Debug, Deserialize)]
struct ServiceArgs {
    service: String,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<McpSearchResult>,
}

#[derive(Debug, Serialize)]
struct RecordResponse {
    record: Option<McpRecordDetail>,
}

#[derive(Debug, Serialize)]
struct TagsResponse {
    tags: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct McpSearchResult {
    key: Option<String>,
    title: String,
    tags: Vec<String>,
    service: Option<String>,
    env: Option<String>,
    snippet: String,
    match_kind: String,
}

#[derive(Debug, Serialize)]
struct McpRecordDetail {
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

impl<'a, S: Storage> ReadOnlyMcp<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        Self { storage }
    }

    pub fn tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: SEARCH_CONTEXT,
                description: "Search context records by keyword or phrase.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "integer", "minimum": 1, "maximum": 50}
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: GET_CONTEXT_BY_KEY,
                description: "Get one full context record by key or id.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string"}
                    },
                    "required": ["key"],
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: LIST_TAGS,
                description: "List known tags with record counts.",
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            McpTool {
                name: GET_SERVICE_CONTEXT,
                description: "Get records associated with an exact service name.",
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "service": {"type": "string"},
                        "limit": {"type": "integer", "minimum": 1, "maximum": 50}
                    },
                    "required": ["service"],
                    "additionalProperties": false
                }),
            },
        ]
    }

    pub fn call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        match name {
            SEARCH_CONTEXT => {
                let args: SearchArgs = serde_json::from_value(arguments)?;
                let results = self
                    .storage
                    .search_records(&args.query, normalized_limit(args.limit))?
                    .into_iter()
                    .map(McpSearchResult::from)
                    .collect();
                Ok(serde_json::to_value(SearchResponse { results })?)
            }
            GET_CONTEXT_BY_KEY => {
                let args: KeyArgs = serde_json::from_value(arguments)?;
                let record = self
                    .storage
                    .get_record(&args.key)?
                    .map(McpRecordDetail::from);
                Ok(serde_json::to_value(RecordResponse { record })?)
            }
            LIST_TAGS => {
                let tags = self.storage.list_tags()?;
                Ok(serde_json::to_value(TagsResponse { tags })?)
            }
            GET_SERVICE_CONTEXT => {
                let args: ServiceArgs = serde_json::from_value(arguments)?;
                let results = self
                    .storage
                    .search_by_service(&args.service, normalized_limit(args.limit))?
                    .into_iter()
                    .map(McpSearchResult::from)
                    .collect();
                Ok(serde_json::to_value(SearchResponse { results })?)
            }
            _ => bail!("unknown MCP tool: {name}"),
        }
    }

    pub fn handle_json_rpc(&self, request: Value) -> Result<Option<Value>> {
        let Some(id) = request.get("id").cloned() else {
            return Ok(None);
        };
        let Some(method) = request.get("method").and_then(Value::as_str) else {
            return Ok(Some(json_rpc_error(
                id,
                -32600,
                "JSON-RPC request missing method".to_string(),
            )));
        };

        let response = match method {
            "initialize" => json_rpc_result(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "ctx-hub", "version": env!("CARGO_PKG_VERSION")}
                }),
            ),
            "tools/list" => json_rpc_result(id, json!({"tools": self.tools()})),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
                let Some(name) = params.get("name").and_then(Value::as_str) else {
                    return Ok(Some(json_rpc_error(
                        id,
                        -32602,
                        "tools/call missing tool name".to_string(),
                    )));
                };
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match self.call_tool(name, arguments) {
                    Ok(value) => json_rpc_result(id, tool_result(value)?),
                    Err(err) => json_rpc_error(id, -32602, err.to_string()),
                }
            }
            "shutdown" => json_rpc_result(id, Value::Null),
            _ => json_rpc_error(id, -32601, format!("method not found: {method}")),
        };

        Ok(Some(response))
    }
}

pub fn serve_stdio<S: Storage>(storage: &S) -> Result<()> {
    let handler = ReadOnlyMcp::new(storage);
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handler.handle_json_rpc(request)?,
            Err(err) => Some(json_rpc_error(Value::Null, -32700, err.to_string())),
        };

        if let Some(response) = response {
            serde_json::to_writer(&mut stdout, &response)?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn normalized_limit(limit: Option<usize>) -> usize {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 50)
}

fn tags_from_text(tags_text: &str) -> Vec<String> {
    tags_text
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn tool_result(value: Value) -> Result<Value> {
    let text = serde_json::to_string_pretty(&value)?;
    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": value,
        "isError": false
    }))
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn json_rpc_error(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}

impl From<SearchResult> for McpSearchResult {
    fn from(result: SearchResult) -> Self {
        Self {
            key: result.key,
            title: result.title,
            tags: tags_from_text(&result.tags_text),
            service: result.service,
            env: result.env,
            snippet: result.snippet,
            match_kind: result.match_kind,
        }
    }
}

impl From<RecordDetail> for McpRecordDetail {
    fn from(record: RecordDetail) -> Self {
        Self {
            id: record.id,
            key: record.key,
            title: record.title,
            content: record.content,
            tags: tags_from_text(&record.tags_text),
            service: record.service,
            env: record.env,
            source: record.source,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
