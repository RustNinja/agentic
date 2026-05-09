pub fn selected_str_trim_matches_map_report(raw: &str) -> String {
    str_trim_matches_map_model::selected_str_trim_matches_map(raw)
}

pub fn dead_live_str_trim_matches_map_report(raw: &str) -> String {
    format!("dead-str-trim-matches-map-live-report:{raw}")
}
