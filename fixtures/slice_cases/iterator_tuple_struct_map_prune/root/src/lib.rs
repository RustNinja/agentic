use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_struct_report(raw: &str) -> String {
    tuple_struct_api::selected_tuple_struct_report(raw)
}

pub fn dead_tuple_struct_report(raw: &str) -> String {
    tuple_struct_api::dead_tuple_struct_report(raw)
}
