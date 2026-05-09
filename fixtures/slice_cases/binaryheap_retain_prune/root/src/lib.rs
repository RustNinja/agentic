use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_retain_report(raw: &str) -> String {
    binaryheap_retain_api::selected_binaryheap_retain_report(raw)
}

pub fn dead_binaryheap_retain_report(raw: &str) -> String {
    binaryheap_retain_api::dead_binaryheap_retain_report(raw)
}
