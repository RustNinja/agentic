pub fn selected_option_and_then_report(raw: &str) -> String {
    option_and_model::selected_option_and_then(raw)
}

pub fn dead_live_option_and_then_report(raw: &str) -> String {
    format!("dead-live-option-and:{raw}")
}
