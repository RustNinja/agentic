mod live;

pub use live::selected_hashmap_entry_or_insert_with_report;

pub fn dead_hashmap_entry_or_insert_with_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-with-report:{raw}")
}
