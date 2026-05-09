pub fn selected_result_unwrap_direct_report(raw: &str) -> String {
    result_unwrap_direct_model::selected_result_unwrap_direct(raw)
}

pub fn dead_live_result_unwrap_direct_report(raw: &str) -> String {
    format!("dead-result-unwrap-direct-live-report:{raw}")
}
