use opensourced::opensourced;

#[opensourced]
pub fn selected_nested_struct_report(raw: &str) -> String {
    nested_struct_api::selected_nested_struct_report(raw)
}

pub fn dead_nested_struct_report(raw: &str) -> String {
    nested_struct_api::dead_nested_struct_report(raw)
}
