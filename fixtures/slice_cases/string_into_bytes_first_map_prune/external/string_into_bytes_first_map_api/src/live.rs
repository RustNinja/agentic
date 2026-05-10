pub fn selected_string_into_bytes_first_map_report(raw: &str) -> String {
    string_into_bytes_first_map_model::selected_string_into_bytes_first_map(raw)
}

pub fn dead_live_string_into_bytes_first_map_report(raw: &str) -> String {
    format!("dead-string-into-bytes-first-map-live-report:{raw}")
}
