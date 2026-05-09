pub fn selected_btreeset_replace_report(raw: &str) -> String {
    btreeset_replace_model::selected_btreeset_replace(raw)
}

pub fn dead_live_btreeset_replace_report(raw: &str) -> String {
    format!("dead-btreeset-replace-live-report:{raw}")
}
