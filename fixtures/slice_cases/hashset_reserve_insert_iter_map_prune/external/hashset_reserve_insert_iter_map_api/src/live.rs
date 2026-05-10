pub fn selected_hashset_reserve_insert_iter_map_report(raw: &str) -> String {
    hashset_reserve_insert_iter_map_model::selected_hashset_reserve_insert_iter_map(raw)
}

pub fn dead_live_hashset_reserve_insert_iter_map_report(raw: &str) -> String {
    format!("dead-hashset-reserve-insert-iter-map-live-report:{raw}")
}
