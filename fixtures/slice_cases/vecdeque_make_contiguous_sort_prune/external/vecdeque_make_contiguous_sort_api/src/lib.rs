mod live;

pub use live::selected_vecdeque_make_contiguous_sort_report;

pub fn dead_vecdeque_make_contiguous_sort_report(raw: &str) -> String {
    format!("dead-vecdeque-make-contiguous-sort-report:{raw}")
}
