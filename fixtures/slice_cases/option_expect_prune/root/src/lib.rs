use opensourced::opensourced;

#[opensourced]
pub fn selected_option_expect_report(raw: &str) -> String {
    option_expect_api::selected_option_expect_report(raw)
}

pub fn dead_option_expect_report(raw: &str) -> String {
    option_expect_api::dead_option_expect_report(raw)
}
