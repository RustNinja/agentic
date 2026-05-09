mod live;

pub use live::selected_linkedlist_pop_back_report;

pub fn dead_linkedlist_pop_back_report(raw: &str) -> String {
    format!("dead-linkedlist-pop-back-report:{raw}")
}
