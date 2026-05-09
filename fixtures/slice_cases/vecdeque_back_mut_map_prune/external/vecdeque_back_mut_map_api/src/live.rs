pub fn selected_vecdeque_back_mut_map_report(raw: &str) -> String {
    vecdeque_back_mut_map_model::selected_vecdeque_back_mut_map(raw)
}

pub fn dead_live_vecdeque_back_mut_map_report(raw: &str) -> String {
    format!("dead-vecdeque-back-mut-map-live-report:{raw}")
}
