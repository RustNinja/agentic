pub fn selected_btreemap_into_iter_pairs_report(raw: &str) -> String {
    btreemap_into_iter_pairs_model::selected_btreemap_into_iter_pairs(raw)
}

pub fn dead_live_btreemap_into_iter_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-into-iter-pairs-live-report:{raw}")
}
