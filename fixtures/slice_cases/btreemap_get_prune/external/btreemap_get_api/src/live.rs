pub fn selected_btreemap_get_report(raw: &str) -> String {
    btreemap_get_model::selected_btreemap_get(raw)
}

pub fn dead_live_btreemap_get_report(raw: &str) -> String {
    format!("dead-btreemap-get-live-report:{raw}")
}
