pub fn selected_btreemap_range_pairs_report(raw: &str) -> String {
    btreemap_range_pairs_model::selected_btreemap_range_pairs(raw)
}

pub fn dead_live_btreemap_range_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-range-pairs-live-report:{raw}")
}
