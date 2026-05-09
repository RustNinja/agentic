pub fn selected_last_report(raw: &str) -> String {
    last_model::selected_last(raw)
}

pub fn dead_live_last_report(raw: &str) -> String {
    format!("dead-last-live-report:{raw}")
}
