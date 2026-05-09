use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_range_find_report(raw: &str) -> String {
    btreeset_range_find_api::selected_btreeset_range_find_report(raw)
}

pub fn dead_btreeset_range_find_report(raw: &str) -> String {
    btreeset_range_find_api::dead_btreeset_range_find_report(raw)
}
