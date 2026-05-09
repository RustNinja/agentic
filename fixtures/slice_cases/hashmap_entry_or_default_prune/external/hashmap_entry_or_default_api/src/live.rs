pub fn selected_hashmap_entry_or_default_report(raw: &str) -> String {
    hashmap_entry_or_default_model::selected_hashmap_entry_or_default(raw)
}

pub fn dead_live_hashmap_entry_or_default_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-default-live-report:{raw}")
}
