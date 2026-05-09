pub fn selected_result_err_map_report(raw: &str) -> String {
    result_err_map_model::selected_result_err_map(raw)
}

pub fn dead_live_result_err_map_report(raw: &str) -> String {
    format!("dead-result-err-map-live-report:{raw}")
}
