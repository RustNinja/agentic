use opensourced::opensourced;

#[opensourced]
pub fn selected_result_expect_report(raw: &str) -> String {
    result_expect_api::selected_result_expect_report(raw)
}

pub fn dead_result_expect_report(raw: &str) -> String {
    result_expect_api::dead_result_expect_report(raw)
}
