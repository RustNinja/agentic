pub fn selected_btreeset_difference_report(raw: &str) -> String {
    btreeset_difference_model::selected_btreeset_difference(raw)
}

pub fn dead_live_btreeset_difference_report(raw: &str) -> String {
    format!("dead-live-btreeset-difference:{raw}")
}
