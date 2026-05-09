pub fn selected_btreeset_range_find_report(raw: &str) -> String {
    btreeset_range_find_model::selected_btreeset_range_find(raw)
}

pub fn dead_live_btreeset_range_find_report(raw: &str) -> String {
    format!("dead-btreeset-range-find-live-report:{raw}")
}
