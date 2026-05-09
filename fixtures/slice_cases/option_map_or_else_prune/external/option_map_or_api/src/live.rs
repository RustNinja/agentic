pub fn selected_option_map_or_report(raw: &str) -> String {
    option_map_or_model::selected_option_map_or(raw)
}

pub fn dead_live_option_map_or_report(raw: &str) -> String {
    format!("dead-live-option-map-or:{raw}")
}
