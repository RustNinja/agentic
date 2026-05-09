use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_union_report(raw: &str) -> String {
    btreeset_union_api::selected_btreeset_union_report(raw)
}

pub fn dead_btreeset_union_report(raw: &str) -> String {
    btreeset_union_api::dead_btreeset_union_report(raw)
}
