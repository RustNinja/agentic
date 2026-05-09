pub fn selected_vecdeque_retain_report(raw: &str) -> String {
    vecdeque_retain_model::selected_vecdeque_retain(raw)
}

pub fn dead_live_vecdeque_retain_report(raw: &str) -> String {
    format!("dead-vecdeque-retain-live-report:{raw}")
}
