use opensourced::opensourced;

#[opensourced]
pub fn selected_option_unwrap_direct_report(raw: &str) -> String {
    option_unwrap_direct_api::selected_option_unwrap_direct_report(raw)
}

pub fn dead_option_unwrap_direct_report(raw: &str) -> String {
    option_unwrap_direct_api::dead_option_unwrap_direct_report(raw)
}
