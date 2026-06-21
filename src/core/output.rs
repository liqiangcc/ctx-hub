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
    println!("id: {}", record.id);
    println!("key: {}", record.key.as_deref().unwrap_or_default());
    println!("title: {}", record.title);
    println!("tags: {}", record.tags_text);
    println!("service: {}", record.service.as_deref().unwrap_or_default());
    println!("env: {}", record.env.as_deref().unwrap_or_default());
    println!("source: {}", record.source.as_deref().unwrap_or_default());
    println!("created_at: {}", record.created_at);
    println!("updated_at: {}", record.updated_at);
    println!("\n{}", record.content);
}

pub fn print_tags(tags: &BTreeMap<String, usize>) {
    for (tag, count) in tags {
        println!("{tag}\t{count}");
    }
}
