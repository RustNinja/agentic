pub fn selected_result_unwrap_err_map_report(raw: &str) -> String {
    result_unwrap_err_map_model::selected_result_unwrap_err_map(raw)
}

pub fn dead_live_result_unwrap_err_map_report(raw: &str) -> String {
    format!("dead-live-result-unwrap-err-map-report:{raw}")
}
