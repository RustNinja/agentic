pub fn selected_binaryheap_iter_find_report(raw: &str) -> String {
    binaryheap_iter_find_model::selected_binaryheap_iter_find(raw)
}

pub fn dead_live_binaryheap_iter_find_report(raw: &str) -> String {
    format!("dead-binaryheap-iter-find-live-report:{raw}")
}
