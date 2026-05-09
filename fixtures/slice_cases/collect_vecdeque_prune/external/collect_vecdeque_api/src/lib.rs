mod live;

pub use live::selected_collect_vecdeque_report;

pub fn dead_collect_vecdeque_report(raw: &str) -> String {
    format!("dead-collect-vecdeque-report:{raw}")
}
