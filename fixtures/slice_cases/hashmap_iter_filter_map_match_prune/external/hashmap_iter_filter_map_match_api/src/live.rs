pub fn selected_hashmap_iter_filter_map_match_report(raw: &str) -> String {
    hashmap_iter_filter_map_match_model::selected_hashmap_iter_filter_map_match(raw)
}

pub fn dead_live_hashmap_iter_filter_map_match_report(raw: &str) -> String {
    format!("dead-hashmap-iter-filter-map-match-live-report:{raw}")
}
