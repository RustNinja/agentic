pub fn selected_option_tuple_report(raw: &str) -> String {
    option_tuple_model::selected_option_tuple(raw)
}

pub fn dead_live_option_tuple_report(raw: &str) -> String {
    format!("dead-live-option-tuple:{raw}")
}
