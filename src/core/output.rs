use std::collections::BTreeMap;

use crate::core::record::RecordDetail;
use crate::core::search::SearchResult;

pub fn print_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("no results");
        return;
    }

    for (idx, item) in results.iter().enumerate() {
        println!(
            "[{}] {}",
            idx + 1,
            item.key.as_deref().unwrap_or("<no-key>")
        );
        println!("title: {}", item.title);
        println!("tags: {}", item.tags_text);
        if let Some(service) = &item.service {
            println!("service: {service}");
        }
        if let Some(env) = &item.env {
            println!("env: {env}");
        }
        println!("source: {}", item.match_kind);
        println!("snippet: {}", item.snippet.replace('\n', " "));
        println!();
    }
}

pub fn print_record_detail(record: &RecordDetail) {
    println!("{}", format_record_detail(record));
}

pub fn format_record_detail(record: &RecordDetail) -> String {
    format!(
        "id: {}\nkey: {}\ntitle: {}\ntags: {}\nservice: {}\nenv: {}\nsource: {}\ncreated_at: {}\nupdated_at: {}\n\n{}",
        record.id,
        record.key.as_deref().unwrap_or_default(),
        record.title,
        record.tags_text,
        record.service.as_deref().unwrap_or_default(),
        record.env.as_deref().unwrap_or_default(),
        record.source.as_deref().unwrap_or_default(),
        record.created_at,
        record.updated_at,
        record.content
    )
}

pub fn print_tags(tags: &BTreeMap<String, usize>) {
    for (tag, count) in tags {
        println!("{tag}\t{count}");
    }
}
