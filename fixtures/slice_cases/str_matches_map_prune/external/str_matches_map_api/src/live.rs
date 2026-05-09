pub fn selected_str_matches_map_report(raw: &str) -> String {
    str_matches_map_model::selected_str_matches_map(raw)
}

pub fn dead_live_str_matches_map_report(raw: &str) -> String {
    format!("dead-str-matches-map-live-report:{raw}")
}
