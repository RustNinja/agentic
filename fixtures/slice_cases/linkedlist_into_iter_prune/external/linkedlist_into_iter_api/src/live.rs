pub fn selected_linkedlist_into_iter_report(raw: &str) -> String {
    linkedlist_into_iter_model::selected_linkedlist_into_iter(raw)
}

pub fn dead_live_linkedlist_into_iter_report(raw: &str) -> String {
    format!("dead-linkedlist-into-iter-live-report:{raw}")
}
