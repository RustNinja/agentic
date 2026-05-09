pub fn selected_result_cloned_map_report(raw: &str) -> String {
    result_cloned_map_model::selected_result_cloned_map(raw)
}

pub fn dead_live_result_cloned_map_report(raw: &str) -> String {
    format!("dead-live-result-cloned-map:{raw}")
}
