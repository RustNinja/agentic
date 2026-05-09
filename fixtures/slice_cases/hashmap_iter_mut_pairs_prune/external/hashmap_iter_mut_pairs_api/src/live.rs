pub fn selected_hashmap_iter_mut_pairs_report(raw: &str) -> String {
    hashmap_iter_mut_pairs_model::selected_hashmap_iter_mut_pairs(raw)
}

pub fn dead_live_hashmap_iter_mut_pairs_report(raw: &str) -> String {
    format!("dead-hashmap-iter-mut-pairs-live-report:{raw}")
}
