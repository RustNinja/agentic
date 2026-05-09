pub fn selected_vecdeque_append_iter_report(raw: &str) -> String {
    vecdeque_append_iter_model::selected_vecdeque_append_iter(raw)
}

pub fn dead_live_vecdeque_append_iter_report(raw: &str) -> String {
    format!("dead-live-vecdeque-append-iter:{raw}")
}
