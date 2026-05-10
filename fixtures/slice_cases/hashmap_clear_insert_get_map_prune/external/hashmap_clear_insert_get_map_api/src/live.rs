pub fn selected_hashmap_clear_insert_get_map_report(raw: &str) -> String {
    hashmap_clear_insert_get_map_model::selected_hashmap_clear_insert_get_map(raw)
}

pub fn dead_live_hashmap_clear_insert_get_map_report(raw: &str) -> String {
    format!("dead-hashmap-clear-insert-get-map-live-report:{raw}")
}
