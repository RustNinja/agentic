mod live;

pub use live::selected_linkedlist_pop_front_report;

pub fn dead_linkedlist_pop_front_report(raw: &str) -> String {
    format!("dead-linkedlist-pop-front-report:{raw}")
}
