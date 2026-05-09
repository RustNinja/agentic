use opensourced::opensourced;

#[opensourced]
pub fn selected_result_option_report(raw: &str) -> String {
    result_option_api::selected_result_option_report(raw)
}

pub fn dead_result_option_report(raw: &str) -> String {
    result_option_api::dead_result_option_report(raw)
}
