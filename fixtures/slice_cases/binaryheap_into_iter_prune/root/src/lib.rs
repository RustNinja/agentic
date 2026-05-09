use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_into_iter_report(raw: &str) -> String {
    binaryheap_into_iter_api::selected_binaryheap_into_iter_report(raw)
}

pub fn dead_binaryheap_into_iter_report(raw: &str) -> String {
    binaryheap_into_iter_api::dead_binaryheap_into_iter_report(raw)
}
