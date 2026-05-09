pub fn selected_btreemap_into_keys_map_report(raw: &str) -> String {
    btreemap_into_keys_map_model::selected_btreemap_into_keys_map(raw)
}

pub fn dead_live_btreemap_into_keys_map_report(raw: &str) -> String {
    format!("dead-btreemap-into-keys-map-live-report:{raw}")
}
