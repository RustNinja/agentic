mod live;

pub use live::selected_hashmap_entry_or_insert_report;

pub fn dead_hashmap_entry_or_insert_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-report:{raw}")
}
