pub fn selected_for_btreemap_range_pair_report(raw: &str) -> String {
    for_btreemap_range_pair_model::selected_for_btreemap_range_pair(raw)
}

pub fn dead_live_for_btreemap_range_pair_report(raw: &str) -> String {
    format!("dead-live-for-btreemap-range-pair-report:{raw}")
}
