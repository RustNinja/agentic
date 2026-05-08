use opensourced::opensourced;

#[opensourced]
pub fn selected_error_report(raw: &str) -> String {
    error_api::selected_error_report(raw)
}

pub fn dead_error_report(raw: &str) -> String {
    error_api::dead_error_report(raw)
}
