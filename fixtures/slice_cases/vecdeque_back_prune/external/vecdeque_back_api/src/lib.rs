mod live;

pub use live::selected_vecdeque_back_report;

pub fn dead_vecdeque_back_report(raw: &str) -> String {
    format!("dead-vecdeque-back-report:{raw}")
}
