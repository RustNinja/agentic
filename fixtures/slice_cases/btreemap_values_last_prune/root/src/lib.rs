use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_values_last_report(raw: &str) -> String {
    btreemap_values_last_api::selected_btreemap_values_last_report(raw)
}

pub fn dead_btreemap_values_last_report(raw: &str) -> String {
    btreemap_values_last_api::dead_btreemap_values_last_report(raw)
}
