pub fn selected_max_by_key_report(raw: &str) -> String {
    max_by_key_model::selected_max_by_key(raw)
}

pub fn dead_live_max_by_key_report(raw: &str) -> String {
    format!("dead-max-by-key-live-report:{raw}")
}
