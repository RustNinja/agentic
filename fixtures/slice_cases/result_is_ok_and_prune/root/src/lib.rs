use opensourced::opensourced;

#[opensourced]
pub fn selected_result_ok_check_report(raw: &str) -> String {
    result_ok_check_api::selected_result_ok_check_report(raw)
}

pub fn dead_result_ok_check_report(raw: &str) -> String {
    result_ok_check_api::dead_result_ok_check_report(raw)
}
