use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_filter_report(raw: &str) -> String {
    tuple_filter_api::selected_tuple_filter_report(raw)
}

pub fn dead_tuple_filter_report(raw: &str) -> String {
    tuple_filter_api::dead_tuple_filter_report(raw)
}
