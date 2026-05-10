pub fn selected_cstring_as_bytes_first_map_report(raw: &str) -> String {
    cstring_as_bytes_first_map_model::selected_cstring_as_bytes_first_map(raw)
}

pub fn dead_live_cstring_as_bytes_first_map_report(raw: &str) -> String {
    format!("dead-cstring-as-bytes-first-map-live-report:{raw}")
}
