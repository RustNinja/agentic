pub fn selected_binaryheap_pop_report(raw: &str) -> String {
    binaryheap_pop_model::selected_binaryheap_pop(raw)
}

pub fn dead_live_binaryheap_pop_report(raw: &str) -> String {
    format!("dead-binaryheap-pop-live-report:{raw}")
}
