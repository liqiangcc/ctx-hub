use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(prefix: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "ctx-hub-{prefix}-{}-{nanos}.{extension}",
        std::process::id()
    ))
}

fn run_ctx(db_path: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--db")
        .arg(db_path)
        .args(args)
        .output()
        .expect("failed to run ctx binary");

    if !output.status.success() {
        panic!(
            "ctx command failed\nargs: {:?}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cleanup_sqlite(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

#[test]
fn jsonl_export_import_round_trip() {
    let source_db = temp_path("jsonl-source", "db");
    let imported_db = temp_path("jsonl-imported", "db");
    let export_file = temp_path("jsonl-export", "jsonl");
    let duplicate_file = temp_path("jsonl-duplicate", "jsonl");

    run_ctx(&source_db, &["db", "init"]);
    run_ctx(
        &source_db,
        &[
            "add",
            "--key",
            "runbook.payment.failed",
            "--title",
            "支付失败排查规则",
            "--content",
            "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。",
            "--tag",
            "payment",
            "--tag",
            "runbook",
            "--service",
            "payment-service",
            "--env",
            "test",
        ],
    );
    run_ctx(
        &source_db,
        &[
            "add",
            "--key",
            "command.order.build",
            "--title",
            "order-service 构建命令",
            "--content",
            "mvn clean package -DskipTests -Ptest",
            "--tag",
            "build",
            "--service",
            "order-service",
            "--env",
            "test",
        ],
    );

    let exported = run_ctx(&source_db, &["db", "export", "--format", "jsonl"]);
    let lines = exported.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);

    let mut parsed = Vec::new();
    for line in lines {
        let value = serde_json::from_str::<serde_json::Value>(line).expect("valid jsonl line");
        assert_eq!(value["schema_version"], 1);
        assert!(value["id"].as_str().is_some());
        assert!(value["title"].as_str().is_some());
        assert!(value["content"].as_str().is_some());
        assert!(value["tags"].as_array().is_some());
        parsed.push(value);
    }

    std::fs::write(&export_file, &exported).expect("write exported jsonl");

    let export_path = export_file.to_str().expect("utf-8 temp path");
    let import_output = run_ctx(&imported_db, &["db", "import", export_path]);
    assert!(import_output.contains("imported: 2"));
    assert!(import_output.contains("skipped_duplicates: 0"));

    let imported_info = run_ctx(&imported_db, &["db", "info"]);
    assert!(imported_info.contains("records: 2"));

    let search_output = run_ctx(&imported_db, &["search", "payment_callback_log"]);
    assert!(search_output.contains("runbook.payment.failed"));

    let command_output = run_ctx(&imported_db, &["search", "clean package"]);
    assert!(command_output.contains("command.order.build"));

    let duplicate_output = run_ctx(&imported_db, &["db", "import", export_path]);
    assert!(duplicate_output.contains("imported: 0"));
    assert!(duplicate_output.contains("skipped_duplicates: 2"));

    parsed[0]["id"] = serde_json::Value::String("ctx_duplicate_key_only".to_string());
    let duplicate_key_json = format!("{}\n", serde_json::to_string(&parsed[0]).unwrap());
    std::fs::write(&duplicate_file, duplicate_key_json).expect("write duplicate jsonl");
    let duplicate_path = duplicate_file.to_str().expect("utf-8 temp path");
    let duplicate_key_output = run_ctx(&imported_db, &["db", "import", duplicate_path]);
    assert!(duplicate_key_output.contains("imported: 0"));
    assert!(duplicate_key_output.contains("skipped_duplicates: 1"));

    let final_info = run_ctx(&imported_db, &["db", "info"]);
    assert!(final_info.contains("records: 2"));

    cleanup_sqlite(&source_db);
    cleanup_sqlite(&imported_db);
    let _ = std::fs::remove_file(export_file);
    let _ = std::fs::remove_file(duplicate_file);
}
