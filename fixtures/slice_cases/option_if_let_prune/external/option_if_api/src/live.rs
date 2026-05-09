pub fn selected_option_if_report(raw: &str) -> String {
    option_if_model::selected_option_if(raw)
}

pub fn dead_live_option_if_report(raw: &str) -> String {
    format!("dead-live-option-if:{raw}")
}
