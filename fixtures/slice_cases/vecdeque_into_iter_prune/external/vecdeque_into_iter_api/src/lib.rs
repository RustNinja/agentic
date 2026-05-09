mod live;

pub use live::selected_vecdeque_into_iter_report;

pub fn dead_vecdeque_into_iter_report(raw: &str) -> String {
    format!("dead-vecdeque-into-iter-report:{raw}")
}
