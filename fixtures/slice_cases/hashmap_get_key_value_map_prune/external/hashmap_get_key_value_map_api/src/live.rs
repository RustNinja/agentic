pub fn selected_hashmap_get_key_value_map_report(raw: &str) -> String {
    hashmap_get_key_value_map_model::selected_hashmap_get_key_value_map(raw)
}

pub fn dead_live_hashmap_get_key_value_map_report(raw: &str) -> String {
    format!("dead-hashmap-get-key-value-map-live-report:{raw}")
}
