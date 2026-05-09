use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_replace_report(raw: &str) -> String {
    btreeset_replace_api::selected_btreeset_replace_report(raw)
}

pub fn dead_btreeset_replace_report(raw: &str) -> String {
    btreeset_replace_api::dead_btreeset_replace_report(raw)
}
