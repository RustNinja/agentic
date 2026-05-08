pub fn selected_option_inspect_report(raw: &str) -> String {
    option_inspect_model::selected_option_inspect(raw)
}

pub fn dead_live_option_inspect_report(raw: &str) -> String {
    format!("dead-live-option-inspect:{raw}")
}
