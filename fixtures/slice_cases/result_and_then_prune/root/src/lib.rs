use opensourced::opensourced;

#[opensourced]
pub fn selected_result_and_report(raw: &str) -> String {
    result_and_api::selected_result_and_report(raw)
}

pub fn dead_result_and_report(raw: &str) -> String {
    result_and_api::dead_result_and_report(raw)
}
