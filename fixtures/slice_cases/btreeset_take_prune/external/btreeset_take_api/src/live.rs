pub fn selected_btreeset_take_report(raw: &str) -> String {
    btreeset_take_model::selected_btreeset_take(raw)
}

pub fn dead_live_btreeset_take_report(raw: &str) -> String {
    format!("dead-btreeset-take-live-report:{raw}")
}
