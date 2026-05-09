pub fn selected_collect_btreemap_report(raw: &str) -> String {
    collect_btreemap_model::selected_collect_btreemap(raw)
}

pub fn dead_live_collect_btreemap_report(raw: &str) -> String {
    format!("dead-collect-btreemap-live-report:{raw}")
}
