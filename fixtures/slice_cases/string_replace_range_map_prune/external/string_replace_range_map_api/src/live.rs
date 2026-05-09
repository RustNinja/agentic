pub fn selected_string_replace_range_map_report(raw: &str) -> String {
    string_replace_range_map_model::selected_string_replace_range_map(raw)
}

pub fn dead_live_string_replace_range_map_report(raw: &str) -> String {
    format!("dead-string-replace-range-map-live-report:{raw}")
}
