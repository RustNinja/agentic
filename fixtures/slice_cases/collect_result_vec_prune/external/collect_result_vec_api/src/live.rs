pub fn selected_collect_result_vec_report(raw: &str) -> String {
    collect_result_vec_model::selected_collect_result_vec(raw)
}

pub fn dead_live_collect_result_vec_report(raw: &str) -> String {
    format!("dead-collect-result-vec-live-report:{raw}")
}
