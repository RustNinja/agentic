pub fn selected_hashmap_drain_pairs_report(raw: &str) -> String {
    hashmap_drain_pairs_model::selected_hashmap_drain_pairs(raw)
}

pub fn dead_live_hashmap_drain_pairs_report(raw: &str) -> String {
    format!("dead-hashmap-drain-pairs-live-report:{raw}")
}
