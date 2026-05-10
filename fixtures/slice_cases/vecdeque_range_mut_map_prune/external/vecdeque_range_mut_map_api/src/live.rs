pub fn selected_vecdeque_range_mut_map_report(raw: &str) -> String {
    vecdeque_range_mut_map_model::selected_vecdeque_range_mut_map(raw)
}

pub fn dead_live_vecdeque_range_mut_map_report(raw: &str) -> String {
    format!("dead-live-vecdeque_range_mut_map-report:{raw}")
}
