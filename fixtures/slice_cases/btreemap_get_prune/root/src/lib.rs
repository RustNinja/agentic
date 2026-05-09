use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_get_report(raw: &str) -> String {
    btreemap_get_api::selected_btreemap_get_report(raw)
}

pub fn dead_btreemap_get_report(raw: &str) -> String {
    btreemap_get_api::dead_btreemap_get_report(raw)
}
