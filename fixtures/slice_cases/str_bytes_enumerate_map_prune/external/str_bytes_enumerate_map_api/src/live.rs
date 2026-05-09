pub fn selected_str_bytes_enumerate_map_report(raw: &str) -> String {
    str_bytes_enumerate_map_model::selected_str_bytes_enumerate_map(raw)
}

pub fn dead_live_str_bytes_enumerate_map_report(raw: &str) -> String {
    format!("dead-str-bytes-enumerate-map-live-report:{raw}")
}
