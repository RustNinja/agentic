pub fn selected_btreeset_pop_first_report(raw: &str) -> String {
    btreeset_pop_first_model::selected_btreeset_pop_first(raw)
}

pub fn dead_live_btreeset_pop_first_report(raw: &str) -> String {
    format!("dead-btreeset-pop-first-live-report:{raw}")
}
