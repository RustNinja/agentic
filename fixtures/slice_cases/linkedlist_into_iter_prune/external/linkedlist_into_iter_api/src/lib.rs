mod live;

pub use live::selected_linkedlist_into_iter_report;

pub fn dead_linkedlist_into_iter_report(raw: &str) -> String {
    format!("dead-linkedlist-into-iter-report:{raw}")
}
