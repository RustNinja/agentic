pub fn selected_binaryheap_drain_report(raw: &str) -> String {
    binaryheap_drain_model::selected_binaryheap_drain(raw)
}

pub fn dead_live_binaryheap_drain_report(raw: &str) -> String {
    format!("dead-binaryheap-drain-live-report:{raw}")
}
