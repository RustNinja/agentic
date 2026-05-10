pub fn selected_for_hashmap_iter_pair_report(raw: &str) -> String {
    for_hashmap_iter_pair_model::selected_for_hashmap_iter_pair(raw)
}

pub fn dead_live_for_hashmap_iter_pair_report(raw: &str) -> String {
    format!("dead-live-for-hashmap-iter-pair-report:{raw}")
}
