pub fn selected_option_report(raw: &str) -> String {
    option_model::selected_option(raw)
}

pub fn dead_live_option_report(raw: &str) -> String {
    format!("dead-live-option:{raw}")
}
