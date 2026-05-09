pub fn selected_option_flatten_report(raw: &str) -> String {
    option_flatten_model::selected_option_flatten(raw)
}

pub fn dead_live_option_flatten_report(raw: &str) -> String {
    format!("dead-option-flatten-live-report:{raw}")
}
