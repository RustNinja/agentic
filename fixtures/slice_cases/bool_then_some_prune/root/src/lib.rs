use opensourced::opensourced;

#[opensourced]
pub fn selected_bool_then_some_report(raw: &str) -> String {
    bool_then_some_api::selected_bool_then_some_report(raw)
}

pub fn dead_bool_then_some_report(raw: &str) -> String {
    bool_then_some_api::dead_bool_then_some_report(raw)
}
