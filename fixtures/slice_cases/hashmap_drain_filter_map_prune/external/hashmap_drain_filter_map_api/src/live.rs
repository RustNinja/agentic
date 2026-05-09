pub fn selected_hashmap_drain_filter_map_report(raw: &str) -> String {
    hashmap_drain_filter_map_model::selected_hashmap_drain_filter_map(raw)
}

pub fn dead_live_hashmap_drain_filter_map_report(raw: &str) -> String {
    format!("dead-hashmap-drain-filter-map-live-report:{raw}")
}
