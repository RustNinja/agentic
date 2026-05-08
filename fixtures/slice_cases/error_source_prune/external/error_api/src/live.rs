pub fn selected_error_report(raw: &str) -> String {
    error_model::selected_error(raw)
}

pub fn dead_live_error_report(raw: &str) -> String {
    format!("dead-live-error:{raw}")
}
