pub fn selected_vecdeque_push_front_back_map_report(raw: &str) -> String {
    vecdeque_push_front_back_map_model::selected_vecdeque_push_front_back_map(raw)
}

pub fn dead_live_vecdeque_push_front_back_map_report(raw: &str) -> String {
    format!("dead-vecdeque-push-front-back-map-live-report:{raw}")
}
