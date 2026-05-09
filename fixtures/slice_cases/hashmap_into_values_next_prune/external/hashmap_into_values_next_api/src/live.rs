pub fn selected_hashmap_into_values_next_report(raw: &str) -> String {
    hashmap_into_values_next_model::selected_hashmap_into_values_next(raw)
}

pub fn dead_live_hashmap_into_values_next_report(raw: &str) -> String {
    format!("dead-hashmap-into-values-next-live-report:{raw}")
}
