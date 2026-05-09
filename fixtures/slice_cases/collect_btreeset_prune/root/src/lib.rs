use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_btreeset_report(raw: &str) -> String {
    collect_btreeset_api::selected_collect_btreeset_report(raw)
}

pub fn dead_collect_btreeset_report(raw: &str) -> String {
    collect_btreeset_api::dead_collect_btreeset_report(raw)
}
