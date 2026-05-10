pub fn selected_string_extend_chars_map_report(raw: &str) -> String {
    string_extend_chars_map_model::selected_string_extend_chars_map(raw)
}

pub fn dead_live_string_extend_chars_map_report(raw: &str) -> String {
    format!("dead-string-extend-chars-map-live-report:{raw}")
}
