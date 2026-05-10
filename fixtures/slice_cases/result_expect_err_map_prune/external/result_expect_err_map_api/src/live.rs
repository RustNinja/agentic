pub fn selected_result_expect_err_map_report(raw: &str) -> String {
    result_expect_err_map_model::selected_result_expect_err_map(raw)
}

pub fn dead_live_result_expect_err_map_report(raw: &str) -> String {
    format!("dead-live-result-expect-err-map-report:{raw}")
}
