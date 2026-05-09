use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_difference_report(raw: &str) -> String {
    btreeset_difference_api::selected_btreeset_difference_report(raw)
}

pub fn dead_btreeset_difference_report(raw: &str) -> String {
    btreeset_difference_api::dead_btreeset_difference_report(raw)
}
