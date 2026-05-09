use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_retain_report(raw: &str) -> String {
    btreemap_retain_api::selected_btreemap_retain_report(raw)
}

pub fn dead_btreemap_retain_report(raw: &str) -> String {
    btreemap_retain_api::dead_btreemap_retain_report(raw)
}
