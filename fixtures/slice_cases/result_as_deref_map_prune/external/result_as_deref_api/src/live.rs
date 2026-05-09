pub fn selected_result_as_deref_map_report(raw: &str) -> String {
    result_as_deref_model::selected_result_as_deref_map(raw)
}

pub fn dead_live_result_as_deref_map_report(raw: &str) -> String {
    format!("dead-live-result-as-deref-map:{raw}")
}
