pub fn selected_rposition_report(raw: &str) -> String {
    rposition_model::selected_rposition(raw)
}

pub fn dead_live_rposition_report(raw: &str) -> String {
    format!("dead-rposition-live-report:{raw}")
}
