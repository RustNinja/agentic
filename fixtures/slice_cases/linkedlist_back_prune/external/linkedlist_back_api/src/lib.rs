mod live;

pub use live::selected_linkedlist_back_report;

pub fn dead_linkedlist_back_report(raw: &str) -> String {
    format!("dead-linkedlist-back-report:{raw}")
}
