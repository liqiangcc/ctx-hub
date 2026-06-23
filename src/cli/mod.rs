use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::core::jsonl::{read_jsonl, write_jsonl};
use crate::core::output::{format_record_detail, print_record_detail, print_results, print_tags};
use crate::core::record::RecordInput;
use crate::mcp;
use crate::storage::sqlite::SqliteStorage;

#[derive(Parser, Debug)]
#[command(name = "ctx")]
#[command(about = "Personal context hub backed by local SQLite search")]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "SQLite database path"
    )]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Manage the local SQLite database")]
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
    #[command(about = "Add a context record")]
    Add {
        #[arg(long, help = "Short human-readable title")]
        title: String,
        #[arg(long, help = "Record body, command, URL, or runbook text")]
        content: String,
        #[arg(long, help = "Stable lookup key, such as runbook.payment.failed")]
        key: Option<String>,
        #[arg(long = "tag", help = "Repeatable tag")]
        tags: Vec<String>,
        #[arg(long, help = "Related service name")]
        service: Option<String>,
        #[arg(long, help = "Related environment")]
        env: Option<String>,
        #[arg(long = "source", help = "Source note for where the context came from")]
        origin: Option<String>,
    },
    #[command(about = "Search context records")]
    Search {
        #[arg(help = "Search keyword or phrase")]
        query: String,
        #[arg(long, default_value_t = 10, help = "Maximum number of results")]
        limit: usize,
    },
    #[command(about = "Show one context record by key or id")]
    Show {
        #[arg(value_name = "KEY_OR_ID")]
        key_or_id: String,
    },
    #[command(about = "List records with an exact tag")]
    Tag {
        #[arg(help = "Exact tag to list")]
        tag: String,
        #[arg(long, default_value_t = 10, help = "Maximum number of results")]
        limit: usize,
    },
    #[command(about = "List known tags with counts")]
    ListTags,
    #[command(about = "Copy record content to the clipboard")]
    Copy {
        #[arg(value_name = "KEY_OR_ID")]
        key_or_id: String,
        #[arg(long, value_enum, default_value_t = CopyField::Content)]
        field: CopyField,
        #[arg(long, help = "Print the selected value instead of using the clipboard")]
        print: bool,
    },
    #[command(about = "Run the read-only MCP server over stdio")]
    Mcp,
}

#[derive(Subcommand, Debug)]
enum DbCommand {
    #[command(about = "Create database tables and search indexes")]
    Init,
    #[command(about = "Show database path and record count")]
    Info,
    #[command(about = "Rebuild SQLite FTS indexes")]
    RebuildIndex,
    #[command(about = "Export active records")]
    Export {
        #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
        format: ExportFormat,
    },
    #[command(about = "Import records from JSONL")]
    Import {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CopyField {
    Content,
    Command,
    Url,
    Key,
    Title,
    Full,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ExportFormat {
    Jsonl,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let storage = SqliteStorage::open(cli.db.as_ref())?;

    match cli.command {
        Commands::Db { command } => run_db_command(&storage, command)?,
        Commands::Add {
            title,
            content,
            key,
            tags,
            service,
            env,
            origin,
        } => {
            storage.init()?;
            let record = RecordInput::new(title, content, key, tags, service, env, origin)?;
            storage.insert_record(&record)?;
            println!("added: {}", record.key.as_deref().unwrap_or(&record.id));
        }
        Commands::Search { query, limit } => {
            storage.init()?;
            let results = storage.search_records(&query, limit)?;
            print_results(&results);
        }
        Commands::Show { key_or_id } => {
            storage.init()?;
            if let Some(record) = storage.get_record(&key_or_id)? {
                print_record_detail(&record);
            } else {
                bail!("record not found: {key_or_id}");
            }
        }
        Commands::Tag { tag, limit } => {
            storage.init()?;
            let results = storage.search_by_tag(&tag, limit)?;
            print_results(&results);
        }
        Commands::ListTags => {
            storage.init()?;
            let tags = storage.list_tags()?;
            print_tags(&tags);
        }
        Commands::Copy {
            key_or_id,
            field,
            print,
        } => {
            storage.init()?;
            copy_record(&storage, &key_or_id, field, print)?;
        }
        Commands::Mcp => {
            storage.init()?;
            mcp::serve_stdio(&storage)?;
        }
    }

    Ok(())
}

fn run_db_command(storage: &SqliteStorage, command: DbCommand) -> Result<()> {
    match command {
        DbCommand::Init => {
            storage.init()?;
            println!("initialized: {}", storage.path().display());
        }
        DbCommand::Info => {
            storage.init()?;
            println!("db: {}", storage.path().display());
            println!("records: {}", storage.record_count()?);
        }
        DbCommand::RebuildIndex => {
            storage.init()?;
            storage.rebuild_index()?;
            println!("fts indexes rebuilt");
        }
        DbCommand::Export { format } => {
            storage.init()?;
            match format {
                ExportFormat::Jsonl => {
                    let records = storage.export_jsonl_records()?;
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    write_jsonl(&records, &mut stdout)?;
                }
            }
        }
        DbCommand::Import { file } => {
            storage.init()?;
            let file_reader = File::open(&file)
                .with_context(|| format!("failed to open import file: {}", file.display()))?;
            let records = read_jsonl(BufReader::new(file_reader))?;
            let summary = storage.import_jsonl_records(records)?;
            println!("imported: {}", summary.imported);
            println!("skipped_duplicates: {}", summary.skipped_duplicates);
        }
    }

    Ok(())
}

fn copy_record(
    storage: &SqliteStorage,
    key_or_id: &str,
    field: CopyField,
    print: bool,
) -> Result<()> {
    let record = storage
        .get_record(key_or_id)?
        .with_context(|| format!("record not found: {key_or_id}"))?;
    let value = copy_value(&record, field);

    if print {
        println!("{value}");
        return Ok(());
    }

    match copy_to_clipboard(&value) {
        Ok(()) => {
            println!(
                "copied: {} ({})",
                record.key.as_deref().unwrap_or(&record.id),
                field_name(field)
            );
        }
        Err(err) => {
            eprintln!("clipboard unavailable: {err}");
            eprintln!("printing selected value instead");
            println!("{value}");
        }
    }

    Ok(())
}

fn copy_value(record: &crate::core::record::RecordDetail, field: CopyField) -> String {
    match field {
        CopyField::Content | CopyField::Command | CopyField::Url => record.content.clone(),
        CopyField::Key => record.key.clone().unwrap_or_else(|| record.id.clone()),
        CopyField::Title => record.title.clone(),
        CopyField::Full => format_record_detail(record),
    }
}

fn field_name(field: CopyField) -> &'static str {
    match field {
        CopyField::Content => "content",
        CopyField::Command => "command",
        CopyField::Url => "url",
        CopyField::Key => "key",
        CopyField::Title => "title",
        CopyField::Full => "full",
    }
}

fn copy_to_clipboard(value: &str) -> Result<()> {
    if let Ok(command) = std::env::var("CTX_HUB_COPY_CMD") {
        return run_clipboard_command(&command, &[], value);
    }

    #[cfg(target_os = "macos")]
    {
        run_clipboard_command("pbcopy", &[], value)
    }

    #[cfg(target_os = "windows")]
    {
        run_clipboard_command("cmd", &["/C", "clip"], value)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        try_clipboard_commands(
            &[
                ("wl-copy", &[][..]),
                ("xclip", &["-selection", "clipboard"][..]),
                ("xsel", &["--clipboard", "--input"][..]),
            ],
            value,
        )
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn try_clipboard_commands(commands: &[(&str, &[&str])], value: &str) -> Result<()> {
    let mut errors = Vec::new();
    for (program, args) in commands {
        match run_clipboard_command(program, args, value) {
            Ok(()) => return Ok(()),
            Err(err) => errors.push(err.to_string()),
        }
    }
    bail!("no clipboard command succeeded: {}", errors.join("; "))
}

fn run_clipboard_command(program: &str, args: &[&str], value: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start clipboard command `{program}`"))?;

    child
        .stdin
        .as_mut()
        .context("failed to open clipboard command stdin")?
        .write_all(value.as_bytes())
        .with_context(|| format!("failed to write to clipboard command `{program}`"))?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to wait for clipboard command `{program}`"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!(
        "clipboard command `{program}` exited with status {}: {}",
        output.status,
        stderr.trim()
    )
}
