pub fn selected_option_copied_map_report(raw: &str) -> String {
    option_copied_model::selected_option_copied_map(raw)
}

pub fn dead_live_option_copied_map_report(raw: &str) -> String {
    format!("dead-live-option-copied-map:{raw}")
}
