pub fn selected_btreemap_first_key_value_report(raw: &str) -> String {
    btreemap_first_key_value_model::selected_btreemap_first_key_value(raw)
}

pub fn dead_live_btreemap_first_key_value_report(raw: &str) -> String {
    format!("dead-live-btreemap-first-key-value:{raw}")
}
