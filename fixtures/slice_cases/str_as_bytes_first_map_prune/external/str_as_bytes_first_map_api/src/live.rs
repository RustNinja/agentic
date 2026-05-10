pub fn selected_str_as_bytes_first_map_report(raw: &str) -> String {
    str_as_bytes_first_map_model::selected_str_as_bytes_first_map(raw)
}

pub fn dead_live_str_as_bytes_first_map_report(raw: &str) -> String {
    format!("dead-str-as-bytes-first-map-live-report:{raw}")
}
