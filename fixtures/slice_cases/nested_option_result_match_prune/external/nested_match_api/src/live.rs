pub fn selected_nested_match_report(raw: &str) -> String {
    nested_match_model::selected_nested_match(raw)
}

pub fn dead_live_nested_match_report(raw: &str) -> String {
    format!("dead-live-nested-match:{raw}")
}
