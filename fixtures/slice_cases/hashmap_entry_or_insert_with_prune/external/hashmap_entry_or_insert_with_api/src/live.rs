pub fn selected_hashmap_entry_or_insert_with_report(raw: &str) -> String {
    hashmap_entry_or_insert_with_model::selected_hashmap_entry_or_insert_with(raw)
}

pub fn dead_live_hashmap_entry_or_insert_with_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-insert-with-live-report:{raw}")
}
