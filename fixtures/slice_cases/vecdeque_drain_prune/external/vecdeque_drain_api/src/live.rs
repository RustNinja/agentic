pub fn selected_vecdeque_drain_report(raw: &str) -> String {
    vecdeque_drain_model::selected_vecdeque_drain(raw)
}

pub fn dead_live_vecdeque_drain_report(raw: &str) -> String {
    format!("dead-vecdeque-drain-live-report:{raw}")
}
