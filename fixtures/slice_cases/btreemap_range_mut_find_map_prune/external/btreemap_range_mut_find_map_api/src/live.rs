pub fn selected_btreemap_range_mut_find_map_report(raw: &str) -> String {
    btreemap_range_mut_find_map_model::selected_btreemap_range_mut_find_map(raw)
}

pub fn dead_live_btreemap_range_mut_find_map_report(raw: &str) -> String {
    format!("dead-btreemap-range-mut-find-map-live-report:{raw}")
}
