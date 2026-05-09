pub fn selected_result_option_if_report(raw: &str) -> String {
    result_option_if_model::selected_result_option_if(raw)
}

pub fn dead_live_result_option_if_report(raw: &str) -> String {
    format!("dead-live-result-option-if:{raw}")
}
