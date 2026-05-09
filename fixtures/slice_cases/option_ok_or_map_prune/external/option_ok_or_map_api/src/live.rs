pub fn selected_option_ok_or_map_report(raw: &str) -> String {
    option_ok_or_map_model::selected_option_ok_or_map(raw)
}

pub fn dead_live_option_ok_or_map_report(raw: &str) -> String {
    format!("dead-live-option-ok-or-map:{raw}")
}
