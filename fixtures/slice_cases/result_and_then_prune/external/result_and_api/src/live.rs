pub fn selected_result_and_report(raw: &str) -> String {
    result_and_model::selected_result_and(raw)
}

pub fn dead_live_result_and_report(raw: &str) -> String {
    format!("dead-live-result-and:{raw}")
}
