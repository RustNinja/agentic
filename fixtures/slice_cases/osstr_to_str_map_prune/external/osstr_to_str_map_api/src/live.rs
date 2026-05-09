pub fn selected_osstr_to_str_map_report(raw: &str) -> String {
    osstr_to_str_map_model::selected_osstr_to_str_map(raw)
}

pub fn dead_live_osstr_to_str_map_report(raw: &str) -> String {
    format!("dead-osstr-to-str-map-live-report:{raw}")
}
