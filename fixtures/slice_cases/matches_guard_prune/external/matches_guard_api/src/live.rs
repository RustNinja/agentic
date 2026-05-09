pub fn selected_matches_guard_report(raw: &str) -> String {
    matches_guard_model::selected_matches_guard(raw)
}

pub fn dead_live_matches_guard_report(raw: &str) -> String {
    format!("dead-live-matches-guard:{raw}")
}
