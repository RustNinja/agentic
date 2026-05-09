pub fn selected_linkedlist_back_report(raw: &str) -> String {
    linkedlist_back_model::selected_linkedlist_back(raw)
}

pub fn dead_live_linkedlist_back_report(raw: &str) -> String {
    format!("dead-linkedlist-back-live-report:{raw}")
}
