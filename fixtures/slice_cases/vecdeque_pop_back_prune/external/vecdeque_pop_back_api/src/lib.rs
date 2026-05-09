mod live;

pub use live::selected_vecdeque_pop_back_report;

pub fn dead_vecdeque_pop_back_report(raw: &str) -> String {
    format!("dead-vecdeque-pop-back-report:{raw}")
}
