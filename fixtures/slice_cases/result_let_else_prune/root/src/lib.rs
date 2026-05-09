use opensourced::opensourced;

#[opensourced]
pub fn selected_result_let_report(raw: &str) -> String {
    result_let_api::selected_result_let_report(raw)
}

pub fn dead_result_let_report(raw: &str) -> String {
    result_let_api::dead_result_let_report(raw)
}
