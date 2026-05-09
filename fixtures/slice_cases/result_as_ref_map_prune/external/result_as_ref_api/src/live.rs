pub fn selected_result_as_ref_map_report(raw: &str) -> String {
    result_as_ref_model::selected_result_as_ref_map(raw)
}

pub fn dead_live_result_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-result-as-ref-map:{raw}")
}
