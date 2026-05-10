pub fn selected_string_retain_map_report(raw: &str) -> String {
    string_retain_map_model::selected_string_retain_map(raw)
}

pub fn dead_live_string_retain_map_report(raw: &str) -> String {
    format!("dead-string-retain-map-live-report:{raw}")
}
