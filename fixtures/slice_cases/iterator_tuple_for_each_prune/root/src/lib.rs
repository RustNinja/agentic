use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_for_each_report(raw: &str) -> String {
    tuple_for_each_api::selected_tuple_for_each_report(raw)
}

pub fn dead_tuple_for_each_report(raw: &str) -> String {
    tuple_for_each_api::dead_tuple_for_each_report(raw)
}
