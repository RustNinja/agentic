use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_pop_first_report(raw: &str) -> String {
    btreeset_pop_first_api::selected_btreeset_pop_first_report(raw)
}

pub fn dead_btreeset_pop_first_report(raw: &str) -> String {
    btreeset_pop_first_api::dead_btreeset_pop_first_report(raw)
}
