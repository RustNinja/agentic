pub fn selected_option_take_if_map_report(raw: &str) -> String {
    option_take_if_map_model::selected_option_take_if_map(raw)
}

pub fn dead_live_option_take_if_map_report(raw: &str) -> String {
    format!("dead-live-option-take-if-map:{raw}")
}
