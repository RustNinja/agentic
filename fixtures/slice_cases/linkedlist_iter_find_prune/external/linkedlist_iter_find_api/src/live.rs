pub fn selected_linkedlist_iter_find_report(raw: &str) -> String {
    linkedlist_iter_find_model::selected_linkedlist_iter_find(raw)
}

pub fn dead_live_linkedlist_iter_find_report(raw: &str) -> String {
    format!("dead-linkedlist-iter-find-live-report:{raw}")
}
