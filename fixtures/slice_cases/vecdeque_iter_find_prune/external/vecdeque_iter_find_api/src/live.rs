pub fn selected_vecdeque_iter_find_report(raw: &str) -> String {
    vecdeque_iter_find_model::selected_vecdeque_iter_find(raw)
}

pub fn dead_live_vecdeque_iter_find_report(raw: &str) -> String {
    format!("dead-vecdeque-iter-find-live-report:{raw}")
}
