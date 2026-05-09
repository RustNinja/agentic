pub fn selected_hashmap_values_next_report(raw: &str) -> String {
    hashmap_values_next_model::selected_hashmap_values_next(raw)
}

pub fn dead_live_hashmap_values_next_report(raw: &str) -> String {
    format!("dead-hashmap-values-next-live-report:{raw}")
}
