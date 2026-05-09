mod live;

pub use live::selected_vecdeque_iter_find_report;

pub fn dead_vecdeque_iter_find_report(raw: &str) -> String {
    format!("dead-vecdeque-iter-find-report:{raw}")
}
