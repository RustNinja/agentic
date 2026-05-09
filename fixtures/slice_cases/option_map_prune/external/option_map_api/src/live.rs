pub fn selected_option_map_report(raw: &str) -> String {
    option_map_model::selected_option_map(raw)
}

pub fn dead_live_option_map_report(raw: &str) -> String {
    format!("dead-option-map-live-report:{raw}")
}
