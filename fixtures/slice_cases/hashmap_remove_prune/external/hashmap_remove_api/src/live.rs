pub fn selected_hashmap_remove_report(raw: &str) -> String {
    hashmap_remove_model::selected_hashmap_remove(raw)
}

pub fn dead_live_hashmap_remove_report(raw: &str) -> String {
    format!("dead-hashmap-remove-live-report:{raw}")
}
