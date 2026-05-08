use opensourced::opensourced;

#[opensourced]
pub fn selected_unwrap_report(raw: &str) -> String {
    unwrap_api::selected_unwrap_report(raw)
}

pub fn dead_unwrap_report(raw: &str) -> String {
    unwrap_api::dead_unwrap_report(raw)
}
