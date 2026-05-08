pub fn selected_entry_map_report(raw: &str) -> String {
    entry_map_model::selected_entry_map(raw)
}

pub fn dead_live_entry_map_report(raw: &str) -> String {
    format!("dead-live-entry-map:{raw}")
}
