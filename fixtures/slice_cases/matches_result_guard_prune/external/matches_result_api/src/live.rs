pub fn selected_matches_result_report(raw: &str) -> String {
    matches_result_model::selected_matches_result(raw)
}

pub fn dead_live_matches_result_report(raw: &str) -> String {
    format!("dead-live-matches-result:{raw}")
}
