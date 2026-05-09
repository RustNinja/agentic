pub fn selected_btreeset_union_report(raw: &str) -> String {
    btreeset_union_model::selected_btreeset_union(raw)
}

pub fn dead_live_btreeset_union_report(raw: &str) -> String {
    format!("dead-live-btreeset-union:{raw}")
}
