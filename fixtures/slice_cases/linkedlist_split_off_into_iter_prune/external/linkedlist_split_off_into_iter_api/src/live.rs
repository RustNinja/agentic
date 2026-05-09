pub fn selected_linkedlist_split_off_into_iter_report(raw: &str) -> String {
    linkedlist_split_off_into_iter_model::selected_linkedlist_split_off_into_iter(raw)
}

pub fn dead_live_linkedlist_split_off_into_iter_report(raw: &str) -> String {
    format!("dead-live-linkedlist-split-off-into-iter:{raw}")
}
