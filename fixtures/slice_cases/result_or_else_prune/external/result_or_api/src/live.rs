pub fn selected_result_or_report(raw: &str) -> String {
    result_or_model::selected_result_or(raw)
}

pub fn dead_live_result_or_report(raw: &str) -> String {
    format!("dead-live-result-or:{raw}")
}
