pub fn selected_btreemap_range_mut_pairs_report(raw: &str) -> String {
    btreemap_range_mut_pairs_model::selected_btreemap_range_mut_pairs(raw)
}

pub fn dead_live_btreemap_range_mut_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-range-mut-pairs-live-report:{raw}")
}
