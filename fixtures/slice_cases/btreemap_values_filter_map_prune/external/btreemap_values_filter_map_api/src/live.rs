pub fn selected_btreemap_values_filter_map_report(raw: &str) -> String {
    btreemap_values_filter_map_model::selected_btreemap_values_filter_map(raw)
}

pub fn dead_live_btreemap_values_filter_map_report(raw: &str) -> String {
    format!("dead-btreemap-values-filter-map-live-report:{raw}")
}
