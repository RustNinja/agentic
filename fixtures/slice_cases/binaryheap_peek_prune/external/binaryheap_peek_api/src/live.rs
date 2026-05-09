pub fn selected_binaryheap_peek_report(raw: &str) -> String {
    binaryheap_peek_model::selected_binaryheap_peek(raw)
}

pub fn dead_live_binaryheap_peek_report(raw: &str) -> String {
    format!("dead-binaryheap-peek-live-report:{raw}")
}
