pub fn selected_vecdeque_truncate_front_map_report(raw: &str) -> String {
    vecdeque_truncate_front_map_model::selected_vecdeque_truncate_front_map(raw)
}

pub fn dead_live_vecdeque_truncate_front_map_report(raw: &str) -> String {
    format!("dead-vecdeque-truncate-front-map-live-report:{raw}")
}
