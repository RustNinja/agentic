pub fn selected_btreemap_entry_remove_map_report(raw: &str) -> String {
    btreemap_entry_remove_map_model::selected_btreemap_entry_remove_map(raw)
}

pub fn dead_live_btreemap_entry_remove_map_report(raw: &str) -> String {
    format!("dead-btreemap-entry-remove-map-live-report:{raw}")
}
