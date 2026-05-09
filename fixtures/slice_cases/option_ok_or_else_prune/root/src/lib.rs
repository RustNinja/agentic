use opensourced::opensourced;

#[opensourced]
pub fn selected_option_ok_report(raw: &str) -> String {
    option_ok_api::selected_option_ok_report(raw)
}

pub fn dead_option_ok_report(raw: &str) -> String {
    option_ok_api::dead_option_ok_report(raw)
}
