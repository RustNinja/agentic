pub fn selected_option_ok_report(raw: &str) -> String {
    option_ok_model::selected_option_ok(raw)
}

pub fn dead_live_option_ok_report(raw: &str) -> String {
    format!("dead-live-option-ok:{raw}")
}
