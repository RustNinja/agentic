pub fn selected_vecdeque_front_mut_map_report(raw: &str) -> String {
    vecdeque_front_mut_map_model::selected_vecdeque_front_mut_map(raw)
}

pub fn dead_live_vecdeque_front_mut_map_report(raw: &str) -> String {
    format!("dead-vecdeque-front-mut-map-live-report:{raw}")
}
