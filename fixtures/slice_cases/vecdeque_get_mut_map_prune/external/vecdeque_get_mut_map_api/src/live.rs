pub fn selected_vecdeque_get_mut_map_report(raw: &str) -> String {
    vecdeque_get_mut_map_model::selected_vecdeque_get_mut_map(raw)
}

pub fn dead_live_vecdeque_get_mut_map_report(raw: &str) -> String {
    format!("dead-vecdeque-get-mut-map-live-report:{raw}")
}
