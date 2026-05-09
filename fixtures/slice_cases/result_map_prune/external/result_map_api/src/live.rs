pub fn selected_result_map_report(raw: &str) -> String {
    result_map_model::selected_result_map(raw)
}

pub fn dead_live_result_map_report(raw: &str) -> String {
    format!("dead-result-map-live-report:{raw}")
}
