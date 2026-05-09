pub fn selected_btreemap_keys_map_report(raw: &str) -> String {
    btreemap_keys_map_model::selected_btreemap_keys_map(raw)
}

pub fn dead_live_btreemap_keys_map_report(raw: &str) -> String {
    format!("dead-btreemap-keys-map-live-report:{raw}")
}
