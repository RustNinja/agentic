pub fn selected_hashmap_remove_entry_map_report(raw: &str) -> String {
    hashmap_remove_entry_map_model::selected_hashmap_remove_entry_map(raw)
}

pub fn dead_live_hashmap_remove_entry_map_report(raw: &str) -> String {
    format!("dead-hashmap-remove-entry-map-live-report:{raw}")
}
