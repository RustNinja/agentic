pub fn selected_option_or_report(raw: &str) -> String {
    option_or_model::selected_option_or(raw)
}

pub fn dead_live_option_or_report(raw: &str) -> String {
    format!("dead-live-option-or:{raw}")
}
