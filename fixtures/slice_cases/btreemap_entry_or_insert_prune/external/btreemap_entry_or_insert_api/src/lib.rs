mod live;

pub use live::selected_btreemap_entry_or_insert_report;

pub fn dead_btreemap_entry_or_insert_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-insert-report:{raw}")
}
