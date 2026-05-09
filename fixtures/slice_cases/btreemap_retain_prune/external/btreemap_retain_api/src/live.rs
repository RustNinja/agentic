pub fn selected_btreemap_retain_report(raw: &str) -> String {
    btreemap_retain_model::selected_btreemap_retain(raw)
}

pub fn dead_live_btreemap_retain_report(raw: &str) -> String {
    format!("dead-btreemap-retain-live-report:{raw}")
}
