use opensourced::opensourced;

#[opensourced]
pub fn selected_option_if_report(raw: &str) -> String {
    option_if_api::selected_option_if_report(raw)
}

pub fn dead_option_if_report(raw: &str) -> String {
    option_if_api::dead_option_if_report(raw)
}
