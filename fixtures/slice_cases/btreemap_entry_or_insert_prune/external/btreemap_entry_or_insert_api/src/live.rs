pub fn selected_btreemap_entry_or_insert_report(raw: &str) -> String {
    btreemap_entry_or_insert_model::selected_btreemap_entry_or_insert(raw)
}

pub fn dead_live_btreemap_entry_or_insert_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-insert-live-report:{raw}")
}
