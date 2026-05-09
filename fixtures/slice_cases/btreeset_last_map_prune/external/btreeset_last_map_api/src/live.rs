pub fn selected_btreeset_last_map_report(raw: &str) -> String {
    btreeset_last_map_model::selected_btreeset_last_map(raw)
}

pub fn dead_live_btreeset_last_map_report(raw: &str) -> String {
    format!("dead-btreeset-last-map-live-report:{raw}")
}
