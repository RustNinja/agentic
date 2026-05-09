pub fn selected_btreemap_remove_entry_map_report(raw: &str) -> String {
    btreemap_remove_entry_map_model::selected_btreemap_remove_entry_map(raw)
}

pub fn dead_live_btreemap_remove_entry_map_report(raw: &str) -> String {
    format!("dead-btreemap-remove-entry-map-live-report:{raw}")
}
