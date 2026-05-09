pub fn selected_result_transpose_unwrap_map_report(raw: &str) -> String {
    result_transpose_unwrap_map_model::selected_result_transpose_unwrap_map(raw)
}

pub fn dead_live_result_transpose_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-result-transpose-unwrap-map:{raw}")
}
