pub fn selected_result_unwrap_or_value_report(raw: &str) -> String {
    result_unwrap_or_value_model::selected_result_unwrap_or_value(raw)
}

pub fn dead_live_result_unwrap_or_value_report(raw: &str) -> String {
    format!("dead-result-unwrap-or-value-live-report:{raw}")
}
