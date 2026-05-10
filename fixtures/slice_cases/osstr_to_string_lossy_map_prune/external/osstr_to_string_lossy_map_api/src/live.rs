pub fn selected_osstr_to_string_lossy_map_report(raw: &str) -> String {
    osstr_to_string_lossy_map_model::selected_osstr_to_string_lossy_map(raw)
}

pub fn dead_live_osstr_to_string_lossy_map_report(raw: &str) -> String {
    format!("dead-osstr-to-string-lossy-map-live-report:{raw}")
}
