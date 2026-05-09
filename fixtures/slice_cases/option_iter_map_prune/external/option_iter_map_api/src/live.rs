pub fn selected_option_iter_map_report(raw: &str) -> String {
    option_iter_map_model::selected_option_iter_map(raw)
}

pub fn dead_live_option_iter_map_report(raw: &str) -> String {
    format!("dead-live-option-iter-map:{raw}")
}
