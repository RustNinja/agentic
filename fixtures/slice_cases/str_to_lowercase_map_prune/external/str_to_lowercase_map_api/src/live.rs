pub fn selected_str_to_lowercase_map_report(raw: &str) -> String {
    str_to_lowercase_map_model::selected_str_to_lowercase_map(raw)
}

pub fn dead_live_str_to_lowercase_map_report(raw: &str) -> String {
    format!("dead-str-to-lowercase-map-live-report:{raw}")
}
