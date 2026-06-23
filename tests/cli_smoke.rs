use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "ctx-hub-cli-smoke-{}-{nanos}.db",
        std::process::id()
    ))
}

fn run_ctx(db_path: &PathBuf, args: &[&str]) -> String {
    let output = run_ctx_output(db_path, args);

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

fn run_ctx_output(db_path: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--db")
        .arg(db_path)
        .args(args)
        .output()
        .expect("failed to run ctx binary")
}

fn run_ctx_output_with_env(db_path: &PathBuf, args: &[&str], key: &str, value: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--db")
        .arg(db_path)
        .args(args)
        .env(key, value)
        .output()
        .expect("failed to run ctx binary")
}

fn run_ctx_failure(db_path: &PathBuf, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--db")
        .arg(db_path)
        .args(args)
        .output()
        .expect("failed to run ctx binary");

    if output.status.success() {
        panic!(
            "ctx command unexpectedly succeeded\nargs: {:?}\nstdout:\n{}\nstderr:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn sqlite_fts_cli_smoke_test() {
    let db_path = temp_db_path();

    let init_output = run_ctx(&db_path, &["db", "init"]);
    assert!(init_output.contains("initialized"));

    let empty_info_output = run_ctx(&db_path, &["db", "info"]);
    assert!(empty_info_output.contains("db:"));
    assert!(empty_info_output.contains("records: 0"));

    let add_runbook_output = run_ctx(
        &db_path,
        &[
            "add",
            "--key",
            "runbook.payment.failed",
            "--title",
            "支付失败排查规则",
            "--content",
            "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。错误码 401 需要检查 mock token。",
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
    assert!(add_runbook_output.contains("runbook.payment.failed"));

    let add_command_output = run_ctx(
        &db_path,
        &[
            "add",
            "--key",
            "command.order.build",
            "--title",
            "order-service 构建命令",
            "--content",
            "mvn clean package -DskipTests -Ptest",
            "--tag",
            "order-service",
            "--tag",
            "build",
            "--service",
            "order-service",
            "--env",
            "test",
        ],
    );
    assert!(add_command_output.contains("command.order.build"));

    let zh_result = run_ctx(&db_path, &["search", "支付"]);
    assert!(zh_result.contains("runbook.payment.failed"));

    let en_result = run_ctx(&db_path, &["search", "payment"]);
    assert!(en_result.contains("runbook.payment.failed"));

    let service_result = run_ctx(&db_path, &["search", "order-service"]);
    assert!(service_result.contains("command.order.build"));

    let command_result = run_ctx(&db_path, &["search", "clean package"]);
    assert!(command_result.contains("command.order.build"));

    let show_result = run_ctx(&db_path, &["show", "runbook.payment.failed"]);
    assert!(show_result.contains("支付失败排查规则"));
    assert!(show_result.contains("payment_callback_log"));

    let tag_result = run_ctx(&db_path, &["tag", "runbook"]);
    assert!(tag_result.contains("runbook.payment.failed"));

    let list_tags_result = run_ctx(&db_path, &["list-tags"]);
    assert!(list_tags_result.contains("payment"));
    assert!(list_tags_result.contains("runbook"));

    let copy_content = run_ctx(&db_path, &["copy", "runbook.payment.failed", "--print"]);
    assert!(copy_content.contains("payment_callback_log"));
    assert!(copy_content.contains("payment-service"));

    let fallback_copy = run_ctx_output_with_env(
        &db_path,
        &["copy", "runbook.payment.failed"],
        "CTX_HUB_COPY_CMD",
        "ctx-hub-missing-copy-command",
    );
    assert!(fallback_copy.status.success());
    assert!(String::from_utf8_lossy(&fallback_copy.stdout).contains("payment_callback_log"));
    assert!(String::from_utf8_lossy(&fallback_copy.stderr).contains("clipboard unavailable"));

    let copy_key = run_ctx(
        &db_path,
        &[
            "copy",
            "runbook.payment.failed",
            "--field",
            "key",
            "--print",
        ],
    );
    assert_eq!(copy_key.trim(), "runbook.payment.failed");

    let copy_title = run_ctx(
        &db_path,
        &[
            "copy",
            "runbook.payment.failed",
            "--field",
            "title",
            "--print",
        ],
    );
    assert_eq!(copy_title.trim(), "支付失败排查规则");

    let copy_command = run_ctx(
        &db_path,
        &[
            "copy",
            "command.order.build",
            "--field",
            "command",
            "--print",
        ],
    );
    assert!(copy_command.contains("mvn clean package"));

    let copy_full = run_ctx(
        &db_path,
        &[
            "copy",
            "runbook.payment.failed",
            "--field",
            "full",
            "--print",
        ],
    );
    assert!(copy_full.contains("title: 支付失败排查规则"));
    assert!(copy_full.contains("payment_callback_log"));

    let rebuild_output = run_ctx(&db_path, &["db", "rebuild-index"]);
    assert!(rebuild_output.contains("fts indexes rebuilt"));

    let final_info_output = run_ctx(&db_path, &["db", "info"]);
    assert!(final_info_output.contains("records: 2"));

    let copy_help = run_ctx(&db_path, &["copy", "--help"]);
    assert!(copy_help.contains("Copy record content to the clipboard"));
    assert!(copy_help.contains("--field"));

    let missing_show_error = run_ctx_failure(&db_path, &["show", "missing.record"]);
    assert!(missing_show_error.contains("record not found: missing.record"));

    let missing_copy_error = run_ctx_failure(&db_path, &["copy", "missing.record", "--print"]);
    assert!(missing_copy_error.contains("record not found: missing.record"));

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
}
