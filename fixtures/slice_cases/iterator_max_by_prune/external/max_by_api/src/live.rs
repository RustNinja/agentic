pub fn selected_max_by_report(raw: &str) -> String {
    max_by_model::selected_max_by(raw)
}

pub fn dead_live_max_by_report(raw: &str) -> String {
    format!("dead-max-by-live-report:{raw}")
}
