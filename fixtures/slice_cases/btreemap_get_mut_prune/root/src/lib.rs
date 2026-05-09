use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_get_mut_report(raw: &str) -> String {
    btreemap_get_mut_api::selected_btreemap_get_mut_report(raw)
}

pub fn dead_btreemap_get_mut_report(raw: &str) -> String {
    btreemap_get_mut_api::dead_btreemap_get_mut_report(raw)
}
