pub fn selected_enum_match_report(raw: &str) -> String {
    enum_match_model::selected_enum_match(raw)
}

pub fn dead_live_enum_match_report(raw: &str) -> String {
    format!("dead-live-enum-match:{raw}")
}
