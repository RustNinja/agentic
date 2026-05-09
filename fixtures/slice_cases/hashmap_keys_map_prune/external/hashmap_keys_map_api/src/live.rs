pub fn selected_hashmap_keys_map_report(raw: &str) -> String {
    hashmap_keys_map_model::selected_hashmap_keys_map(raw)
}

pub fn dead_live_hashmap_keys_map_report(raw: &str) -> String {
    format!("dead-hashmap-keys-map-live-report:{raw}")
}
