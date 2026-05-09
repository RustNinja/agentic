pub fn selected_collect_vecdeque_report(raw: &str) -> String {
    collect_vecdeque_model::selected_collect_vecdeque(raw)
}

pub fn dead_live_collect_vecdeque_report(raw: &str) -> String {
    format!("dead-collect-vecdeque-live-report:{raw}")
}
