use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_if_let_report(raw: &str) -> String {
    enum_if_let_api::selected_enum_if_let_report(raw)
}

pub fn dead_enum_if_let_report(raw: &str) -> String {
    enum_if_let_api::dead_enum_if_let_report(raw)
}
