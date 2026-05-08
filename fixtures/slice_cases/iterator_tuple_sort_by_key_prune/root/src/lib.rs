use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_sort_by_key_report(raw: &str) -> String {
    tuple_sort_key_api::selected_tuple_sort_by_key_report(raw)
}

pub fn dead_tuple_sort_by_key_report(raw: &str) -> String {
    tuple_sort_key_api::dead_tuple_sort_by_key_report(raw)
}
