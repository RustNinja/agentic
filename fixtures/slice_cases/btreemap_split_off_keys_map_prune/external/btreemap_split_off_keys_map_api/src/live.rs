pub fn selected_btreemap_split_off_keys_map_report(raw: &str) -> String {
    btreemap_split_off_keys_map_model::selected_btreemap_split_off_keys_map(raw)
}

pub fn dead_live_btreemap_split_off_keys_map_report(raw: &str) -> String {
    format!("dead-btreemap-split-off-keys-map-live-report:{raw}")
}
