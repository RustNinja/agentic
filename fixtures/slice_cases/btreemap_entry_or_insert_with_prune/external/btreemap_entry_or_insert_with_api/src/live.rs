pub fn selected_btreemap_entry_or_insert_with_report(raw: &str) -> String {
    btreemap_entry_or_insert_with_model::selected_btreemap_entry_or_insert_with(raw)
}

pub fn dead_live_btreemap_entry_or_insert_with_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-insert-with-live-report:{raw}")
}
