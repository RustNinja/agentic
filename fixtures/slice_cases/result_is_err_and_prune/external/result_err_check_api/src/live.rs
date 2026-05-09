pub fn selected_result_err_check_report(raw: &str) -> String {
    result_err_check_model::selected_result_err_check(raw)
}

pub fn dead_live_result_err_check_report(raw: &str) -> String {
    format!("dead-live-result-err-check:{raw}")
}
