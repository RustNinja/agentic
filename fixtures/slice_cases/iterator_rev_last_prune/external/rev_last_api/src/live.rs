pub fn selected_rev_last_report(raw: &str) -> String {
    rev_last_model::selected_rev_last(raw)
}

pub fn dead_live_rev_last_report(raw: &str) -> String {
    format!("dead-rev-last-live-report:{raw}")
}
