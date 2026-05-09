mod live;

pub use live::selected_vecdeque_pop_front_report;

pub fn dead_vecdeque_pop_front_report(raw: &str) -> String {
    format!("dead-vecdeque-pop-front-report:{raw}")
}
