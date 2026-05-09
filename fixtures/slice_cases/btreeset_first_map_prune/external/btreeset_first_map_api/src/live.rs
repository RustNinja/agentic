pub fn selected_btreeset_first_map_report(raw: &str) -> String {
    btreeset_first_map_model::selected_btreeset_first_map(raw)
}

pub fn dead_live_btreeset_first_map_report(raw: &str) -> String {
    format!("dead-btreeset-first-map-live-report:{raw}")
}
