use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_annotated_btreeset_report(raw: &str) -> String {
    collect_annotated_btreeset_api::selected_collect_annotated_btreeset_report(raw)
}

pub fn dead_collect_annotated_btreeset_report(raw: &str) -> String {
    collect_annotated_btreeset_api::dead_live_collect_annotated_btreeset(raw)
}
