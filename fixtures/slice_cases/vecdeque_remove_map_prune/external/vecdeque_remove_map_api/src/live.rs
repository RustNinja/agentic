pub fn selected_vecdeque_remove_map_report(raw: &str) -> String {
    vecdeque_remove_map_model::selected_vecdeque_remove_map(raw)
}

pub fn dead_live_vecdeque_remove_map_report(raw: &str) -> String {
    format!("dead-vecdeque-remove-map-live-report:{raw}")
}
