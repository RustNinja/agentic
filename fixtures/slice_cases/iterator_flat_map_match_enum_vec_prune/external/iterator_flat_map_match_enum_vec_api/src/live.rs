pub fn selected_iterator_flat_map_match_enum_vec_report(raw: &str) -> String {
    iterator_flat_map_match_enum_vec_model::selected_iterator_flat_map_match_enum_vec(raw)
}

pub fn dead_live_iterator_flat_map_match_enum_vec_report(raw: &str) -> String {
    format!("dead-iterator-flat-map-match-enum-vec-live-report:{raw}")
}
