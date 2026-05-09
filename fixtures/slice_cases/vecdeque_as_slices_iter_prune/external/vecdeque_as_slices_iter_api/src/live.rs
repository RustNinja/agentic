pub fn selected_vecdeque_as_slices_iter_report(raw: &str) -> String {
    vecdeque_as_slices_iter_model::selected_vecdeque_as_slices_iter(raw)
}

pub fn dead_live_vecdeque_as_slices_iter_report(raw: &str) -> String {
    format!("dead-live-vecdeque-as-slices-iter:{raw}")
}
