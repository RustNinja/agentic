pub fn selected_vecdeque_get_map_report(raw: &str) -> String {
    vecdeque_get_map_model::selected_vecdeque_get_map(raw)
}

pub fn dead_live_vecdeque_get_map_report(raw: &str) -> String {
    format!("dead-vecdeque-get-map-live-report:{raw}")
}
