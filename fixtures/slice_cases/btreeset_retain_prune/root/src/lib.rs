use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_retain_report(raw: &str) -> String {
    btreeset_retain_api::selected_btreeset_retain_report(raw)
}

pub fn dead_btreeset_retain_report(raw: &str) -> String {
    btreeset_retain_api::dead_btreeset_retain_report(raw)
}
