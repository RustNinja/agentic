pub fn selected_hashmap_values_filter_map_report(raw: &str) -> String {
    hashmap_values_filter_map_model::selected_hashmap_values_filter_map(raw)
}

pub fn dead_live_hashmap_values_filter_map_report(raw: &str) -> String {
    format!("dead-hashmap-values-filter-map-live-report:{raw}")
}
