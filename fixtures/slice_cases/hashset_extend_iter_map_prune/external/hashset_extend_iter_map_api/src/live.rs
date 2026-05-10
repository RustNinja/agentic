pub fn selected_hashset_extend_iter_map_report(raw: &str) -> String {
    hashset_extend_iter_map_model::selected_hashset_extend_iter_map(raw)
}

pub fn dead_live_hashset_extend_iter_map_report(raw: &str) -> String {
    format!("dead-hashset-extend-iter-map-live-report:{raw}")
}
