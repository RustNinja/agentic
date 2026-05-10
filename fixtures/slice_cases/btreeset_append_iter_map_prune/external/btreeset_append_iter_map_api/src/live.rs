pub fn selected_btreeset_append_iter_map_report(raw: &str) -> String {
    btreeset_append_iter_map_model::selected_btreeset_append_iter_map(raw)
}

pub fn dead_live_btreeset_append_iter_map_report(raw: &str) -> String {
    format!("dead-btreeset-append-iter-map-live-report:{raw}")
}
