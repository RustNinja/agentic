use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_remove_report(raw: &str) -> String {
    btreemap_remove_api::selected_btreemap_remove_report(raw)
}

pub fn dead_btreemap_remove_report(raw: &str) -> String {
    btreemap_remove_api::dead_btreemap_remove_report(raw)
}
