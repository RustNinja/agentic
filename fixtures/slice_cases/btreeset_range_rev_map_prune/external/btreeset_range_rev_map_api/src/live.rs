pub fn selected_btreeset_range_rev_map_report(raw: &str) -> String {
    btreeset_range_rev_map_model::selected_btreeset_range_rev_map(raw)
}

pub fn dead_live_btreeset_range_rev_map_report(raw: &str) -> String {
    format!("dead-btreeset-range-rev-map-live-report:{raw}")
}
