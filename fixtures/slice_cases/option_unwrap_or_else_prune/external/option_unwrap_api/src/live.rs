pub fn selected_option_unwrap_report(raw: &str) -> String {
    option_unwrap_model::selected_option_unwrap(raw)
}

pub fn dead_live_option_unwrap_report(raw: &str) -> String {
    format!("dead-live-option-unwrap:{raw}")
}
