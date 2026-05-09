pub fn selected_btreemap_get_mut_report(raw: &str) -> String {
    btreemap_get_mut_model::selected_btreemap_get_mut(raw)
}

pub fn dead_live_btreemap_get_mut_report(raw: &str) -> String {
    format!("dead-btreemap-get-mut-live-report:{raw}")
}
