mod live;

pub use live::selected_btreemap_entry_or_insert_with_key_report;

pub fn dead_btreemap_entry_or_insert_with_key_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-insert-with-key-report:{raw}")
}
