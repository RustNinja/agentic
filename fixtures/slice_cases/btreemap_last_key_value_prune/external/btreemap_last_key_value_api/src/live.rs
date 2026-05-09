pub fn selected_btreemap_last_key_value_report(raw: &str) -> String {
    btreemap_last_key_value_model::selected_btreemap_last_key_value(raw)
}

pub fn dead_live_btreemap_last_key_value_report(raw: &str) -> String {
    format!("dead-live-btreemap-last-key-value:{raw}")
}
