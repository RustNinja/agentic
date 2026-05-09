pub fn selected_btreeset_iter_next_back_map_report(raw: &str) -> String {
    btreeset_iter_next_back_map_model::selected_btreeset_iter_next_back_map(raw)
}

pub fn dead_live_btreeset_iter_next_back_map_report(raw: &str) -> String {
    format!("dead-btreeset-iter-next-back-map-live-report:{raw}")
}
