pub fn selected_result_option_report(raw: &str) -> String {
    result_option_model::selected_result_option(raw)
}

pub fn dead_live_result_option_report(raw: &str) -> String {
    format!("dead-live-result-option:{raw}")
}
