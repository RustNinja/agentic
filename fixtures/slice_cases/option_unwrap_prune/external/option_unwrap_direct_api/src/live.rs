pub fn selected_option_unwrap_direct_report(raw: &str) -> String {
    option_unwrap_direct_model::selected_option_unwrap_direct(raw)
}

pub fn dead_live_option_unwrap_direct_report(raw: &str) -> String {
    format!("dead-option-unwrap-direct-live-report:{raw}")
}
