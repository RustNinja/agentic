use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_drain_report(raw: &str) -> String {
    binaryheap_drain_api::selected_binaryheap_drain_report(raw)
}

pub fn dead_binaryheap_drain_report(raw: &str) -> String {
    binaryheap_drain_api::dead_binaryheap_drain_report(raw)
}
