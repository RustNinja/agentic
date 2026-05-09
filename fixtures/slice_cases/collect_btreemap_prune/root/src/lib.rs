use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_btreemap_report(raw: &str) -> String {
    collect_btreemap_api::selected_collect_btreemap_report(raw)
}

pub fn dead_collect_btreemap_report(raw: &str) -> String {
    collect_btreemap_api::dead_collect_btreemap_report(raw)
}
