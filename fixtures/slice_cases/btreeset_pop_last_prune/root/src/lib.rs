use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_pop_last_report(raw: &str) -> String {
    btreeset_pop_last_api::selected_btreeset_pop_last_report(raw)
}

pub fn dead_btreeset_pop_last_report(raw: &str) -> String {
    btreeset_pop_last_api::dead_btreeset_pop_last_report(raw)
}
