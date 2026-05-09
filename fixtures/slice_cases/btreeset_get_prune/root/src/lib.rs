use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_get_report(raw: &str) -> String {
    btreeset_get_api::selected_btreeset_get_report(raw)
}

pub fn dead_btreeset_get_report(raw: &str) -> String {
    btreeset_get_api::dead_btreeset_get_report(raw)
}
