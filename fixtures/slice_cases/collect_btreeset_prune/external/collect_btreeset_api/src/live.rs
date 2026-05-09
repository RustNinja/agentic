pub fn selected_collect_btreeset_report(raw: &str) -> String {
    collect_btreeset_model::selected_collect_btreeset(raw)
}

pub fn dead_live_collect_btreeset_report(raw: &str) -> String {
    format!("dead-collect-btreeset-live-report:{raw}")
}
