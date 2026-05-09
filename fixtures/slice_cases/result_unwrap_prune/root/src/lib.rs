use opensourced::opensourced;

#[opensourced]
pub fn selected_result_unwrap_direct_report(raw: &str) -> String {
    result_unwrap_direct_api::selected_result_unwrap_direct_report(raw)
}

pub fn dead_result_unwrap_direct_report(raw: &str) -> String {
    result_unwrap_direct_api::dead_result_unwrap_direct_report(raw)
}
