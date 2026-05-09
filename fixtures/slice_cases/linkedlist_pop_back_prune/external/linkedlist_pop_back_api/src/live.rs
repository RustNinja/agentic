pub fn selected_linkedlist_pop_back_report(raw: &str) -> String {
    linkedlist_pop_back_model::selected_linkedlist_pop_back(raw)
}

pub fn dead_live_linkedlist_pop_back_report(raw: &str) -> String {
    format!("dead-linkedlist-pop-back-live-report:{raw}")
}
