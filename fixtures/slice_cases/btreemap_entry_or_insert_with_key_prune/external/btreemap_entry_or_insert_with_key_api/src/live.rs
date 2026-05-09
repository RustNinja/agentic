pub fn selected_btreemap_entry_or_insert_with_key_report(raw: &str) -> String {
    btreemap_entry_or_insert_with_key_model::selected_btreemap_entry_or_insert_with_key(raw)
}

pub fn dead_live_btreemap_entry_or_insert_with_key_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-insert-with-key-live-report:{raw}")
}
