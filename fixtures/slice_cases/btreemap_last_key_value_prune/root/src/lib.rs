use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_last_key_value_report(raw: &str) -> String {
    btreemap_last_key_value_api::selected_btreemap_last_key_value_report(raw)
}

pub fn dead_btreemap_last_key_value_report(raw: &str) -> String {
    btreemap_last_key_value_api::dead_btreemap_last_key_value_report(raw)
}
