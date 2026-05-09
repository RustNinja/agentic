use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_take_report(raw: &str) -> String {
    btreeset_take_api::selected_btreeset_take_report(raw)
}

pub fn dead_btreeset_take_report(raw: &str) -> String {
    btreeset_take_api::dead_btreeset_take_report(raw)
}
