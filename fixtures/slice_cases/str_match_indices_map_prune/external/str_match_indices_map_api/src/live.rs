pub fn selected_str_match_indices_map_report(raw: &str) -> String {
    str_match_indices_map_model::selected_str_match_indices_map(raw)
}

pub fn dead_live_str_match_indices_map_report(raw: &str) -> String {
    format!("dead-str-match-indices-map-live-report:{raw}")
}
