use opensourced::opensourced;

#[opensourced]
pub fn selected_any_report(raw: &str) -> String {
    any_api::selected_any_report(raw)
}

pub fn dead_any_report(raw: &str) -> String {
    any_api::dead_any_report(raw)
}
