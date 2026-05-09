pub fn selected_linkedlist_front_report(raw: &str) -> String {
    linkedlist_front_model::selected_linkedlist_front(raw)
}

pub fn dead_live_linkedlist_front_report(raw: &str) -> String {
    format!("dead-linkedlist-front-live-report:{raw}")
}
