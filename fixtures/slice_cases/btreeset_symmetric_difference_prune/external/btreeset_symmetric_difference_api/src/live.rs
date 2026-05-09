pub fn selected_btreeset_symmetric_difference_report(raw: &str) -> String {
    btreeset_symmetric_difference_model::selected_btreeset_symmetric_difference(raw)
}

pub fn dead_live_btreeset_symmetric_difference_report(raw: &str) -> String {
    format!("dead-live-btreeset-symmetric-difference:{raw}")
}
