pub fn selected_vecdeque_range_map_report(raw: &str) -> String {
    vecdeque_range_map_model::selected_vecdeque_range_map(raw)
}

pub fn dead_live_vecdeque_range_map_report(raw: &str) -> String {
    format!("dead-vecdeque-range-map-live-report:{raw}")
}
