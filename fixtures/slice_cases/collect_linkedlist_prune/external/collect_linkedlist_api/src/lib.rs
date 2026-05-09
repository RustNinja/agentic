mod live;

pub use live::selected_collect_linkedlist_report;

pub fn dead_collect_linkedlist_report(raw: &str) -> String {
    format!("dead-collect-linkedlist-report:{raw}")
}
