pub fn selected_string_split_off_map_report(raw: &str) -> String {
    string_split_off_map_model::selected_string_split_off_map(raw)
}

pub fn dead_live_string_split_off_map_report(raw: &str) -> String {
    format!("dead-string-split-off-map-live-report:{raw}")
}
