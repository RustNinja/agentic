pub fn selected_hashmap_get_report(raw: &str) -> String {
    hashmap_get_model::selected_hashmap_get(raw)
}

pub fn dead_live_hashmap_get_report(raw: &str) -> String {
    format!("dead-hashmap-get-live-report:{raw}")
}
