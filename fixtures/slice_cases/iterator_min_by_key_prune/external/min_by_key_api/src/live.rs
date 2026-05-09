pub fn selected_min_by_key_report(raw: &str) -> String {
    min_by_key_model::selected_min_by_key(raw)
}

pub fn dead_live_min_by_key_report(raw: &str) -> String {
    format!("dead-min-by-key-live-report:{raw}")
}
