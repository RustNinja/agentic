pub fn selected_linkedlist_pop_front_report(raw: &str) -> String {
    linkedlist_pop_front_model::selected_linkedlist_pop_front(raw)
}

pub fn dead_live_linkedlist_pop_front_report(raw: &str) -> String {
    format!("dead-linkedlist-pop-front-live-report:{raw}")
}
