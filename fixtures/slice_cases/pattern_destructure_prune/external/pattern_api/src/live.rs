pub fn selected_pattern_report(raw: &str) -> String {
    pattern_model::selected_pattern(raw)
}

pub fn dead_live_pattern_report(raw: &str) -> String {
    format!("dead-live-pattern:{raw}")
}
