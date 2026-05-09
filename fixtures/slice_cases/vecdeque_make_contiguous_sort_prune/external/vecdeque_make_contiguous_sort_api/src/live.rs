pub fn selected_vecdeque_make_contiguous_sort_report(raw: &str) -> String {
    vecdeque_make_contiguous_sort_model::selected_vecdeque_make_contiguous_sort(raw)
}

pub fn dead_live_vecdeque_make_contiguous_sort_report(raw: &str) -> String {
    format!("dead-vecdeque-make-contiguous-sort-live-report:{raw}")
}
