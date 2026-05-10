pub fn selected_hashmap_extend_values_map_report(raw: &str) -> String {
    hashmap_extend_values_map_model::selected_hashmap_extend_values_map(raw)
}

pub fn dead_live_hashmap_extend_values_map_report(raw: &str) -> String {
    format!("dead-hashmap-extend-values-map-live-report:{raw}")
}
