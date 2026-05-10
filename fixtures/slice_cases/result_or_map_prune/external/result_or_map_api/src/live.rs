pub fn selected_result_or_map_report(raw: &str) -> String {
    result_or_map_model::selected_result_or_map(raw)
}

pub fn dead_live_result_or_map_report(raw: &str) -> String {
    format!("dead-result-or-map-live-report:{raw}")
}
