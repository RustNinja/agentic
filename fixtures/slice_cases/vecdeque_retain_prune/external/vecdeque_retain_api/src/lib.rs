mod live;

pub use live::selected_vecdeque_retain_report;

pub fn dead_vecdeque_retain_report(raw: &str) -> String {
    format!("dead-vecdeque-retain-report:{raw}")
}
