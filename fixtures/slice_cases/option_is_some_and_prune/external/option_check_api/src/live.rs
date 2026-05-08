pub fn selected_option_check_report(raw: &str) -> String {
    option_check_model::selected_option_check(raw)
}

pub fn dead_live_option_check_report(raw: &str) -> String {
    format!("dead-live-option-check:{raw}")
}
