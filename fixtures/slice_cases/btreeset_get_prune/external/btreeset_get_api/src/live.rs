pub fn selected_btreeset_get_report(raw: &str) -> String {
    btreeset_get_model::selected_btreeset_get(raw)
}

pub fn dead_live_btreeset_get_report(raw: &str) -> String {
    format!("dead-btreeset-get-live-report:{raw}")
}
