pub fn selected_iterator_find_map_enum_match_report(raw: &str) -> String {
    iterator_find_map_enum_match_model::selected_iterator_find_map_enum_match(raw)
}

pub fn dead_live_iterator_find_map_enum_match_report(raw: &str) -> String {
    format!("dead-iterator-find-map-enum-match-live-report:{raw}")
}
