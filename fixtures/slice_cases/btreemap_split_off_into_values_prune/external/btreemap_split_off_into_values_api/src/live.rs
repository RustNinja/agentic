pub fn selected_btreemap_split_off_into_values_report(raw: &str) -> String {
    btreemap_split_off_into_values_model::selected_btreemap_split_off_into_values(raw)
}

pub fn dead_live_btreemap_split_off_into_values_report(raw: &str) -> String {
    format!("dead-live-btreemap-split-off-into-values:{raw}")
}
