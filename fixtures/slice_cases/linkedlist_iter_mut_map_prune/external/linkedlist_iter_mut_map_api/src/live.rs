pub fn selected_linkedlist_iter_mut_map_report(raw: &str) -> String {
    linkedlist_iter_mut_map_model::selected_linkedlist_iter_mut_map(raw)
}

pub fn dead_live_linkedlist_iter_mut_map_report(raw: &str) -> String {
    format!("dead-linkedlist-iter-mut-map-live-report:{raw}")
}
