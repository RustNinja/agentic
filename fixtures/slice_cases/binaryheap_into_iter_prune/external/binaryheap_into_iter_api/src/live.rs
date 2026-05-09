pub fn selected_binaryheap_into_iter_report(raw: &str) -> String {
    binaryheap_into_iter_model::selected_binaryheap_into_iter(raw)
}

pub fn dead_live_binaryheap_into_iter_report(raw: &str) -> String {
    format!("dead-binaryheap-into-iter-live-report:{raw}")
}
