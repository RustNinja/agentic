pub fn selected_btreemap_append_keys_map_report(raw: &str) -> String {
    btreemap_append_keys_map_model::selected_btreemap_append_keys_map(raw)
}

pub fn dead_live_btreemap_append_keys_map_report(raw: &str) -> String {
    format!("dead-btreemap-append-keys-map-live-report:{raw}")
}
