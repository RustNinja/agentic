mod live;

pub use live::selected_hashmap_entry_or_insert_with_key_report;

pub fn dead_hashmap_entry_or_insert_with_key_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-with-key-report:{raw}")
}
