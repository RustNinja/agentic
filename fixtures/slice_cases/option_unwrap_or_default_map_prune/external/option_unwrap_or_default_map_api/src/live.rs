pub fn selected_option_unwrap_or_default_map_report(raw: &str) -> String {
    option_unwrap_or_default_map_model::selected_option_unwrap_or_default_map(raw)
}

pub fn dead_live_option_unwrap_or_default_map_report(raw: &str) -> String {
    format!("dead-live-option-unwrap-or-default-map:{raw}")
}
