use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_inspect_report(raw: &str) -> String {
    tuple_inspect_api::selected_tuple_inspect_report(raw)
}

pub fn dead_tuple_inspect_report(raw: &str) -> String {
    tuple_inspect_api::dead_tuple_inspect_report(raw)
}
