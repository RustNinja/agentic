pub fn selected_adjacent_summary(raw: &str) -> String {
    serde_adjacent_model::parse_live_command(raw)
}

pub fn dead_live_adjacent_summary(raw: &str) -> String {
    format!("dead-live-adjacent-summary:{raw}")
}
