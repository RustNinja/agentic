mod live;

pub use live::selected_linkedlist_front_report;

pub fn dead_linkedlist_front_report(raw: &str) -> String {
    format!("dead-linkedlist-front-report:{raw}")
}
