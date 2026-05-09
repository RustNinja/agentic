pub fn selected_result_expect_report(raw: &str) -> String {
    result_expect_model::selected_result_expect(raw)
}

pub fn dead_live_result_expect_report(raw: &str) -> String {
    format!("dead-result-expect-live-report:{raw}")
}
