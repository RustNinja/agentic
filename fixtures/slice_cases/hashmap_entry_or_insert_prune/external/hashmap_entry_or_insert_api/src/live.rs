pub fn selected_hashmap_entry_or_insert_report(raw: &str) -> String {
    hashmap_entry_or_insert_model::selected_hashmap_entry_or_insert(raw)
}

pub fn dead_live_hashmap_entry_or_insert_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-live-report:{raw}")
}
