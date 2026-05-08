use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_report(raw: &str) -> String {
    enum_api::selected_enum_report(raw)
}

pub fn dead_enum_report(raw: &str) -> String {
    enum_api::dead_enum_report(raw)
}
