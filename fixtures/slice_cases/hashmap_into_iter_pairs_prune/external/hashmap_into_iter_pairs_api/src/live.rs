pub fn selected_hashmap_into_iter_pairs_report(raw: &str) -> String {
    hashmap_into_iter_pairs_model::selected_hashmap_into_iter_pairs(raw)
}

pub fn dead_live_hashmap_into_iter_pairs_report(raw: &str) -> String {
    format!("dead-hashmap-into-iter-pairs-live-report:{raw}")
}
