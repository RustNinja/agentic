use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_tuple_report(raw: &str) -> String {
    enum_tuple_api::selected_enum_tuple_report(raw)
}

pub fn dead_enum_tuple_report(raw: &str) -> String {
    enum_tuple_api::dead_enum_tuple_report(raw)
}
