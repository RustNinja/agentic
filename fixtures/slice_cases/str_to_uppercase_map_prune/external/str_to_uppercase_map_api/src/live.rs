pub fn selected_str_to_uppercase_map_report(raw: &str) -> String {
    str_to_uppercase_map_model::selected_str_to_uppercase_map(raw)
}

pub fn dead_live_str_to_uppercase_map_report(raw: &str) -> String {
    format!("dead-str-to-uppercase-map-live-report:{raw}")
}
