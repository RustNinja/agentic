pub fn selected_result_copied_map_report(raw: &str) -> String {
    result_copied_map_model::selected_result_copied_map(raw)
}

pub fn dead_live_result_copied_map_report(raw: &str) -> String {
    format!("dead-live-result-copied-map:{raw}")
}
