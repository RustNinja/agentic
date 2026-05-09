pub fn selected_btreemap_append_values_report(raw: &str) -> String {
    btreemap_append_values_model::selected_btreemap_append_values(raw)
}

pub fn dead_live_btreemap_append_values_report(raw: &str) -> String {
    format!("dead-live-btreemap-append-values:{raw}")
}
