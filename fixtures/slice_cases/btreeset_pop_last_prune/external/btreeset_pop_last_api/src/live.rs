pub fn selected_btreeset_pop_last_report(raw: &str) -> String {
    btreeset_pop_last_model::selected_btreeset_pop_last(raw)
}

pub fn dead_live_btreeset_pop_last_report(raw: &str) -> String {
    format!("dead-btreeset-pop-last-live-report:{raw}")
}
