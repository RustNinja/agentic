use opensourced::opensourced;

#[opensourced]
pub fn selected_option_report(raw: &str) -> String {
    option_api::selected_option_report(raw)
}

pub fn dead_option_report(raw: &str) -> String {
    option_api::dead_option_report(raw)
}
