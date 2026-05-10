pub fn selected_btreemap_append_values_last_map_report(raw: &str) -> String {
    btreemap_append_values_last_map_model::selected_btreemap_append_values_last_map(raw)
}

pub fn dead_live_btreemap_append_values_last_map_report(raw: &str) -> String {
    format!("dead-btreemap-append-values-last-map-live-report:{raw}")
}
