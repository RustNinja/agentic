use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_append_values_report(raw: &str) -> String {
    btreemap_append_values_api::selected_btreemap_append_values_report(raw)
}

pub fn dead_btreemap_append_values_report(raw: &str) -> String {
    btreemap_append_values_api::dead_btreemap_append_values_report(raw)
}
