use opensourced::opensourced;

#[opensourced]
pub fn selected_nested_if_report(raw: &str) -> String {
    nested_if_api::selected_nested_if_report(raw)
}

pub fn dead_nested_if_report(raw: &str) -> String {
    nested_if_api::dead_nested_if_report(raw)
}
