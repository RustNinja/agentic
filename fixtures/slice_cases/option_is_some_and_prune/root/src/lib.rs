use opensourced::opensourced;

#[opensourced]
pub fn selected_option_check_report(raw: &str) -> String {
    option_check_api::selected_option_check_report(raw)
}

pub fn dead_option_check_report(raw: &str) -> String {
    option_check_api::dead_option_check_report(raw)
}
