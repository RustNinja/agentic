pub fn selected_btreemap_remove_report(raw: &str) -> String {
    btreemap_remove_model::selected_btreemap_remove(raw)
}

pub fn dead_live_btreemap_remove_report(raw: &str) -> String {
    format!("dead-btreemap-remove-live-report:{raw}")
}
