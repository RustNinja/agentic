pub fn selected_result_unwrap_or_default_map_report(raw: &str) -> String {
    result_unwrap_or_default_map_model::selected_result_unwrap_or_default_map(raw)
}

pub fn dead_live_result_unwrap_or_default_map_report(raw: &str) -> String {
    format!("dead-live-result-unwrap-or-default-map:{raw}")
}
