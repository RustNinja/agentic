pub fn selected_btreeset_retain_report(raw: &str) -> String {
    btreeset_retain_model::selected_btreeset_retain(raw)
}

pub fn dead_live_btreeset_retain_report(raw: &str) -> String {
    format!("dead-btreeset-retain-live-report:{raw}")
}
