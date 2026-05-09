pub fn selected_cstring_into_string_map_report(raw: &str) -> String {
    cstring_into_string_map_model::selected_cstring_into_string_map(raw)
}

pub fn dead_live_cstring_into_string_map_report(raw: &str) -> String {
    format!("dead-cstring-into-string-map-live-report:{raw}")
}
