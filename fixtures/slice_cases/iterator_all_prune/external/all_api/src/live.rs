pub fn selected_all_report(raw: &str) -> String {
    all_model::selected_all(raw)
}

pub fn dead_live_all_report(raw: &str) -> String {
    format!("dead-all-live-report:{raw}")
}
