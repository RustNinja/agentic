pub fn selected_hashmap_into_keys_map_report(raw: &str) -> String {
    hashmap_into_keys_map_model::selected_hashmap_into_keys_map(raw)
}

pub fn dead_live_hashmap_into_keys_map_report(raw: &str) -> String {
    format!("dead-hashmap-into-keys-map-live-report:{raw}")
}
