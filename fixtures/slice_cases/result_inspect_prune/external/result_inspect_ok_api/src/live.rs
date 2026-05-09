pub fn selected_result_inspect_ok_report(raw: &str) -> String {
    result_inspect_ok_model::selected_result_inspect_ok(raw)
}

pub fn dead_live_result_inspect_ok_report(raw: &str) -> String {
    format!("dead-live-result-inspect-ok:{raw}")
}
