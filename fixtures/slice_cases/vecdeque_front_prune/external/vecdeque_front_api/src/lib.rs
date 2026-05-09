mod live;

pub use live::selected_vecdeque_front_report;

pub fn dead_vecdeque_front_report(raw: &str) -> String {
    format!("dead-vecdeque-front-report:{raw}")
}
