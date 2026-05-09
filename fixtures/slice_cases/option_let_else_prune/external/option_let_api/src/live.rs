pub fn selected_option_let_report(raw: &str) -> String {
    option_let_model::selected_option_let(raw)
}

pub fn dead_live_option_let_report(raw: &str) -> String {
    format!("dead-live-option-let:{raw}")
}
