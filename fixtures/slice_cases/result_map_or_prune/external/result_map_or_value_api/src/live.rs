pub fn selected_result_map_or_value_report(raw: &str) -> String {
    result_map_or_value_model::selected_result_map_or_value(raw)
}

pub fn dead_live_result_map_or_value_report(raw: &str) -> String {
    format!("dead-result-map-or-value-live-report:{raw}")
}
