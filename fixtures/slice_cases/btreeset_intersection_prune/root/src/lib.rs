use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_intersection_report(raw: &str) -> String {
    btreeset_intersection_api::selected_btreeset_intersection_report(raw)
}

pub fn dead_btreeset_intersection_report(raw: &str) -> String {
    btreeset_intersection_api::dead_btreeset_intersection_report(raw)
}
