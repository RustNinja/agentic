pub fn selected_hashmap_shrink_to_fit_values_map_report(raw: &str) -> String {
    hashmap_shrink_to_fit_values_map_model::selected_hashmap_shrink_to_fit_values_map(raw)
}

pub fn dead_live_hashmap_shrink_to_fit_values_map_report(raw: &str) -> String {
    format!("dead-hashmap-shrink-to-fit-values-map-live-report:{raw}")
}
