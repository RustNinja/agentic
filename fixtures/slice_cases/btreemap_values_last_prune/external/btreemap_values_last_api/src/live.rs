pub fn selected_btreemap_values_last_report(raw: &str) -> String {
    btreemap_values_last_model::selected_btreemap_values_last(raw)
}

pub fn dead_live_btreemap_values_last_report(raw: &str) -> String {
    format!("dead-btreemap-values-last-live-report:{raw}")
}
