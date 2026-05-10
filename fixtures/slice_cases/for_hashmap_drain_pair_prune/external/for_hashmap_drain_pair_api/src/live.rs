pub fn selected_for_hashmap_drain_pair_report(raw: &str) -> String {
    for_hashmap_drain_pair_model::selected_for_hashmap_drain_pair(raw)
}

pub fn dead_live_for_hashmap_drain_pair_report(raw: &str) -> String {
    format!("dead-live-for-hashmap-drain-pair-report:{raw}")
}
