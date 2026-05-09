pub fn selected_string_from_utf16_map_report(raw: &str) -> String {
    string_from_utf16_map_model::selected_string_from_utf16_map(raw)
}

pub fn dead_live_string_from_utf16_map_report(raw: &str) -> String {
    format!("dead-string-from-utf16-map-live-report:{raw}")
}
