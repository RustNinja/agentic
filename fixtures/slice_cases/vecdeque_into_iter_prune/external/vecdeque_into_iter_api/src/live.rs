pub fn selected_vecdeque_into_iter_report(raw: &str) -> String {
    vecdeque_into_iter_model::selected_vecdeque_into_iter(raw)
}

pub fn dead_live_vecdeque_into_iter_report(raw: &str) -> String {
    format!("dead-vecdeque-into-iter-live-report:{raw}")
}
