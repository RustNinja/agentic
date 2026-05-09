pub fn selected_result_iter_map_report(raw: &str) -> String {
    result_iter_map_model::selected_result_iter_map(raw)
}

pub fn dead_live_result_iter_map_report(raw: &str) -> String {
    format!("dead-live-result-iter-map:{raw}")
}
