pub fn selected_result_inspect_report(raw: &str) -> String {
    result_inspect_model::selected_result_inspect(raw)
}

pub fn dead_live_result_inspect_report(raw: &str) -> String {
    format!("dead-live-result-inspect:{raw}")
}
