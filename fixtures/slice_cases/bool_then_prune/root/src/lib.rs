use opensourced::opensourced;

#[opensourced]
pub fn selected_bool_then_report(raw: &str) -> String {
    bool_then_api::selected_bool_then_report(raw)
}

pub fn dead_bool_then_report(raw: &str) -> String {
    bool_then_api::dead_bool_then_report(raw)
}
