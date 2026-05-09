pub fn selected_string_from_utf8_map_report(raw: &str) -> String {
    string_from_utf8_map_model::selected_string_from_utf8_map(raw)
}

pub fn dead_live_string_from_utf8_map_report(raw: &str) -> String {
    format!("dead-string-from-utf8-map-live-report:{raw}")
}
