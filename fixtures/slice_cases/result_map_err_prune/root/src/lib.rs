use opensourced::opensourced;

#[opensourced]
pub fn selected_result_report(raw: &str) -> String {
    result_api::selected_result_report(raw)
}

pub fn dead_result_report(raw: &str) -> String {
    result_api::dead_result_report(raw)
}
