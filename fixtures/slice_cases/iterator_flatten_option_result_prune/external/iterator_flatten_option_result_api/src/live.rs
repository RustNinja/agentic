pub fn selected_iterator_flatten_option_result_report(raw: &str) -> String {
    iterator_flatten_option_result_model::selected_iterator_flatten_option_result(raw)
}

pub fn dead_live_iterator_flatten_option_result_report(raw: &str) -> String {
    format!("dead-iterator-flatten-option-result-live-report:{raw}")
}
