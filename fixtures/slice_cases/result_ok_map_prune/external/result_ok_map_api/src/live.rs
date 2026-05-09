pub fn selected_result_ok_map_report(raw: &str) -> String {
    result_ok_map_model::selected_result_ok_map(raw)
}

pub fn dead_live_result_ok_map_report(raw: &str) -> String {
    format!("dead-result-ok-map-live-report:{raw}")
}
