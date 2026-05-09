pub fn selected_iterator_flatten_option_array_report(raw: &str) -> String {
    iterator_flatten_option_array_model::selected_iterator_flatten_option_array(raw)
}

pub fn dead_live_iterator_flatten_option_array_report(raw: &str) -> String {
    format!("dead-iterator-flatten-option-array-live-report:{raw}")
}
