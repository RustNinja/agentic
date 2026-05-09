pub fn selected_option_cloned_map_report(raw: &str) -> String {
    option_cloned_map_model::selected_option_cloned_map(raw)
}

pub fn dead_live_option_cloned_map_report(raw: &str) -> String {
    format!("dead-live-option-cloned-map:{raw}")
}
