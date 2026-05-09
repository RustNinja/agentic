pub fn selected_result_ok_check_report(raw: &str) -> String {
    result_ok_check_model::selected_result_ok_check(raw)
}

pub fn dead_live_result_ok_check_report(raw: &str) -> String {
    format!("dead-live-result-ok-check:{raw}")
}
