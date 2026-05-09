mod live;

pub use live::selected_vecdeque_drain_report;

pub fn dead_vecdeque_drain_report(raw: &str) -> String {
    format!("dead-vecdeque-drain-report:{raw}")
}
