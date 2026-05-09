pub fn selected_hashmap_get_mut_report(raw: &str) -> String {
    hashmap_get_mut_model::selected_hashmap_get_mut(raw)
}

pub fn dead_live_hashmap_get_mut_report(raw: &str) -> String {
    format!("dead-hashmap-get-mut-live-report:{raw}")
}
