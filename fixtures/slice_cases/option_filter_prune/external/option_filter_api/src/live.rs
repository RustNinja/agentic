pub fn selected_option_filter_report(raw: &str) -> String {
    option_filter_model::selected_option_filter(raw)
}

pub fn dead_live_option_filter_report(raw: &str) -> String {
    format!("dead-live-option-filter:{raw}")
}
