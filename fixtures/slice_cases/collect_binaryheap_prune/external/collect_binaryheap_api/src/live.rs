pub fn selected_collect_binaryheap_report(raw: &str) -> String {
    collect_binaryheap_model::selected_collect_binaryheap(raw)
}

pub fn dead_live_collect_binaryheap_report(raw: &str) -> String {
    format!("dead-collect-binaryheap-live-report:{raw}")
}
