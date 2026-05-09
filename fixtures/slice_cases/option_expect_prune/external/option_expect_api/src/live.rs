pub fn selected_option_expect_report(raw: &str) -> String {
    option_expect_model::selected_option_expect(raw)
}

pub fn dead_live_option_expect_report(raw: &str) -> String {
    format!("dead-option-expect-live-report:{raw}")
}
