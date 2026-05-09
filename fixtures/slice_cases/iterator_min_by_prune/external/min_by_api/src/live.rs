pub fn selected_min_by_report(raw: &str) -> String {
    min_by_model::selected_min_by(raw)
}

pub fn dead_live_min_by_report(raw: &str) -> String {
    format!("dead-min-by-live-report:{raw}")
}
