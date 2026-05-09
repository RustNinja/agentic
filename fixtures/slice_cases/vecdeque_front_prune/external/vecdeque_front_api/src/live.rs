pub fn selected_vecdeque_front_report(raw: &str) -> String {
    vecdeque_front_model::selected_vecdeque_front(raw)
}

pub fn dead_live_vecdeque_front_report(raw: &str) -> String {
    format!("dead-vecdeque-front-live-report:{raw}")
}
