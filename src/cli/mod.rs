use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::core::output::{print_record_detail, print_results, print_tags};
use crate::core::record::RecordInput;
use crate::storage::sqlite::SqliteStorage;

#[derive(Parser, Debug)]
#[command(name = "ctx")]
#[command(about = "Context Hub SQLite FTS proof of concept")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
    Add {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long)]
        origin: Option<String>,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Show {
        key_or_id: String,
    },
    Tag {
        tag: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    ListTags,
}

#[derive(Subcommand, Debug)]
enum DbCommand {
    Init,
    Info,
    RebuildIndex,
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
                println!("not found: {key_or_id}");
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
    }

    Ok(())
}
