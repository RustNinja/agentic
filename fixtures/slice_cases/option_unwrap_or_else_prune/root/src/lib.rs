use opensourced::opensourced;

#[opensourced]
pub fn selected_option_unwrap_report(raw: &str) -> String {
    option_unwrap_api::selected_option_unwrap_report(raw)
}

pub fn dead_option_unwrap_report(raw: &str) -> String {
    option_unwrap_api::dead_option_unwrap_report(raw)
}
