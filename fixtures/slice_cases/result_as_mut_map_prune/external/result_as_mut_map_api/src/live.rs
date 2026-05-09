pub fn selected_result_as_mut_map_report(raw: &str) -> String {
    result_as_mut_map_model::selected_result_as_mut_map(raw)
}

pub fn dead_live_result_as_mut_map_report(raw: &str) -> String {
    format!("dead-live-result-as-mut-map:{raw}")
}
