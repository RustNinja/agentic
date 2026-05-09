pub fn selected_btreemap_values_mut_report(raw: &str) -> String {
    btreemap_values_mut_model::selected_btreemap_values_mut(raw)
}

pub fn dead_live_btreemap_values_mut_report(raw: &str) -> String {
    format!("dead-live-btreemap-values-mut:{raw}")
}
