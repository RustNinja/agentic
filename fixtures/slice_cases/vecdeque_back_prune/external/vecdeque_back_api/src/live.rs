pub fn selected_vecdeque_back_report(raw: &str) -> String {
    vecdeque_back_model::selected_vecdeque_back(raw)
}

pub fn dead_live_vecdeque_back_report(raw: &str) -> String {
    format!("dead-vecdeque-back-live-report:{raw}")
}
