pub fn selected_option_unwrap_or_value_report(raw: &str) -> String {
    option_unwrap_or_value_model::selected_option_unwrap_or_value(raw)
}

pub fn dead_live_option_unwrap_or_value_report(raw: &str) -> String {
    format!("dead-option-unwrap-or-value-live-report:{raw}")
}
