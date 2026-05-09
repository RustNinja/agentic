pub fn selected_collect_linkedlist_report(raw: &str) -> String {
    collect_linkedlist_model::selected_collect_linkedlist(raw)
}

pub fn dead_live_collect_linkedlist_report(raw: &str) -> String {
    format!("dead-collect-linkedlist-live-report:{raw}")
}
