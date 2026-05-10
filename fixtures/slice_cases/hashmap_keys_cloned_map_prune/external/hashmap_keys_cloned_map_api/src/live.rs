pub fn selected_hashmap_keys_cloned_map_report(raw: &str) -> String {
    hashmap_keys_cloned_map_model::selected_hashmap_keys_cloned_map(raw)
}

pub fn dead_live_hashmap_keys_cloned_map_report(raw: &str) -> String {
    format!("dead-hashmap-keys-cloned-map-live-report:{raw}")
}
