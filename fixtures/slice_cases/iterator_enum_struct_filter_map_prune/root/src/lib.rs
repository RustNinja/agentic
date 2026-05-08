use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_struct_report(raw: &str) -> String {
    enum_struct_api::selected_enum_struct_report(raw)
}

pub fn dead_enum_struct_report(raw: &str) -> String {
    enum_struct_api::dead_enum_struct_report(raw)
}
