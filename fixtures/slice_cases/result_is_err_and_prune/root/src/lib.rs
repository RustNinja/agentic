use opensourced::opensourced;

#[opensourced]
pub fn selected_result_err_check_report(raw: &str) -> String {
    result_err_check_api::selected_result_err_check_report(raw)
}

pub fn dead_result_err_check_report(raw: &str) -> String {
    result_err_check_api::dead_result_err_check_report(raw)
}
