use opensourced::opensourced;

#[opensourced]
pub fn selected_option_or_report(raw: &str) -> String {
    option_or_api::selected_option_or_report(raw)
}

pub fn dead_option_or_report(raw: &str) -> String {
    option_or_api::dead_option_or_report(raw)
}
