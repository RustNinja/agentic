mod live;

pub use live::selected_linkedlist_iter_find_report;

pub fn dead_linkedlist_iter_find_report(raw: &str) -> String {
    format!("dead-linkedlist-iter-find-report:{raw}")
}
