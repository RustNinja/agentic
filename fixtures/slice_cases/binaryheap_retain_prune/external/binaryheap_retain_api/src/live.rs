pub fn selected_binaryheap_retain_report(raw: &str) -> String {
    binaryheap_retain_model::selected_binaryheap_retain(raw)
}

pub fn dead_live_binaryheap_retain_report(raw: &str) -> String {
    format!("dead-binaryheap-retain-live-report:{raw}")
}
