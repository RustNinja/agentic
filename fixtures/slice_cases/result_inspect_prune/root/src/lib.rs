use opensourced::opensourced;

#[opensourced]
pub fn selected_result_inspect_ok_report(raw: &str) -> String {
    result_inspect_ok_api::selected_result_inspect_ok_report(raw)
}

pub fn dead_result_inspect_ok_report(raw: &str) -> String {
    result_inspect_ok_api::dead_result_inspect_ok_report(raw)
}
