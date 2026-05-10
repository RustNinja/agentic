pub fn selected_hashmap_reserve_insert_values_map_report(raw: &str) -> String {
    hashmap_reserve_insert_values_map_model::selected_hashmap_reserve_insert_values_map(raw)
}

pub fn dead_live_hashmap_reserve_insert_values_map_report(raw: &str) -> String {
    format!("dead-hashmap-reserve-insert-values-map-live-report:{raw}")
}
