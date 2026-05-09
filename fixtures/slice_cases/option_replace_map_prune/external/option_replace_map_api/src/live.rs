pub fn selected_option_replace_map_report(raw: &str) -> String {
    option_replace_map_model::selected_option_replace_map(raw)
}

pub fn dead_live_option_replace_map_report(raw: &str) -> String {
    format!("dead-live-option-replace-map:{raw}")
}
