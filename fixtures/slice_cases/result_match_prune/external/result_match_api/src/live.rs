pub fn selected_result_match_report(raw: &str) -> String {
    result_match_model::selected_result_match(raw)
}

pub fn dead_live_result_match_report(raw: &str) -> String {
    format!("dead-live-result-match:{raw}")
}
