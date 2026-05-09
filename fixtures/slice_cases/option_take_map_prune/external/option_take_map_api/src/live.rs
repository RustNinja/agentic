pub fn selected_option_take_map_report(raw: &str) -> String {
    option_take_map_model::selected_option_take_map(raw)
}

pub fn dead_live_option_take_map_report(raw: &str) -> String {
    format!("dead-live-option-take-map:{raw}")
}
