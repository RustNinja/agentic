pub fn selected_hashmap_retain_report(raw: &str) -> String {
    hashmap_retain_model::selected_hashmap_retain(raw)
}

pub fn dead_live_hashmap_retain_report(raw: &str) -> String {
    format!("dead-hashmap-retain-live-report:{raw}")
}
