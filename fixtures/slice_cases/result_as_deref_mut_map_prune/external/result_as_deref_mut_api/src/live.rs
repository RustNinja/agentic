pub fn selected_result_as_deref_mut_map_report(raw: &str) -> String {
    result_as_deref_mut_model::selected_result_as_deref_mut_map(raw)
}

pub fn dead_live_result_as_deref_mut_map_report(raw: &str) -> String {
    format!("dead-live-result-as-deref-mut-map:{raw}")
}
