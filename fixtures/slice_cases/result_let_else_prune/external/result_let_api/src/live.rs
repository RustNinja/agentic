pub fn selected_result_let_report(raw: &str) -> String {
    result_let_model::selected_result_let(raw)
}

pub fn dead_live_result_let_report(raw: &str) -> String {
    format!("dead-live-result-let:{raw}")
}
