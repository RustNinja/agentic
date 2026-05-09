pub fn selected_option_xor_report(raw: &str) -> String {
    option_xor_model::selected_option_xor(raw)
}

pub fn dead_live_option_xor_report(raw: &str) -> String {
    format!("dead-option-xor-live-report:{raw}")
}
