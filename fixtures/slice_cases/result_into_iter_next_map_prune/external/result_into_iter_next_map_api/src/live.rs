pub fn selected_result_into_iter_next_map_report(raw: &str) -> String {
    result_into_iter_next_map_model::selected_result_into_iter_next_map(raw)
}

pub fn dead_live_result_into_iter_next_map_report(raw: &str) -> String {
    format!("dead-result-into-iter-next-map-live-report:{raw}")
}
