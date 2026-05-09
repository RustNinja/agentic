pub fn selected_hashmap_entry_or_insert_with_key_report(raw: &str) -> String {
    hashmap_entry_or_insert_with_key_model::selected_hashmap_entry_or_insert_with_key(raw)
}

pub fn dead_live_hashmap_entry_or_insert_with_key_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-with-key-live-report:{raw}")
}
