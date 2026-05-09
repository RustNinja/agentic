pub fn selected_result_map_or_report(raw: &str) -> String {
    result_map_or_model::selected_result_map_or(raw)
}

pub fn dead_live_result_map_or_report(raw: &str) -> String {
    format!("dead-live-result-map-or:{raw}")
}
