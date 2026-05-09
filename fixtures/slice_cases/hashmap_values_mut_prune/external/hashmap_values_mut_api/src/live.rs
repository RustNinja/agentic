pub fn selected_hashmap_values_mut_report(raw: &str) -> String {
    hashmap_values_mut_model::selected_hashmap_values_mut(raw)
}

pub fn dead_live_hashmap_values_mut_report(raw: &str) -> String {
    format!("dead-live-hashmap-values-mut:{raw}")
}
