pub fn selected_hashmap_keys_find_report(raw: &str) -> String {
    hashmap_keys_find_model::selected_hashmap_keys_find(raw)
}

pub fn dead_live_hashmap_keys_find_report(raw: &str) -> String {
    format!("dead-hashmap-keys-find-live-report:{raw}")
}
