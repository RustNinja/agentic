pub fn selected_vecdeque_split_off_into_iter_report(raw: &str) -> String {
    vecdeque_split_off_into_iter_model::selected_vecdeque_split_off_into_iter(raw)
}

pub fn dead_live_vecdeque_split_off_into_iter_report(raw: &str) -> String {
    format!("dead-live-vecdeque-split-off-into-iter:{raw}")
}
