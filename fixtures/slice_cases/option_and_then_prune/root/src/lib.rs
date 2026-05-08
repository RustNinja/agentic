use opensourced::opensourced;

#[opensourced]
pub fn selected_option_and_then_report(raw: &str) -> String {
    option_and_api::selected_option_and_then_report(raw)
}

pub fn dead_option_and_then_report(raw: &str) -> String {
    option_and_api::dead_option_and_then_report(raw)
}
