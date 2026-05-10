pub fn selected_string_truncate_map_report(raw: &str) -> String {
    string_truncate_map_model::selected_string_truncate_map(raw)
}

pub fn dead_live_string_truncate_map_report(raw: &str) -> String {
    format!("dead-string-truncate-map-live-report:{raw}")
}
