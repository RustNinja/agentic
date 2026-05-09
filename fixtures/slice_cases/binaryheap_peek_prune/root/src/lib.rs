use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_peek_report(raw: &str) -> String {
    binaryheap_peek_api::selected_binaryheap_peek_report(raw)
}

pub fn dead_binaryheap_peek_report(raw: &str) -> String {
    binaryheap_peek_api::dead_binaryheap_peek_report(raw)
}
