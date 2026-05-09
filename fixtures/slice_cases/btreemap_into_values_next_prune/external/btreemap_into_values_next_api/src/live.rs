pub fn selected_btreemap_into_values_next_report(raw: &str) -> String {
    btreemap_into_values_next_model::selected_btreemap_into_values_next(raw)
}

pub fn dead_live_btreemap_into_values_next_report(raw: &str) -> String {
    format!("dead-btreemap-into-values-next-live-report:{raw}")
}
