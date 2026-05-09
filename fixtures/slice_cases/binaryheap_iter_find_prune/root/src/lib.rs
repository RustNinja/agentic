use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_iter_find_report(raw: &str) -> String {
    binaryheap_iter_find_api::selected_binaryheap_iter_find_report(raw)
}

pub fn dead_binaryheap_iter_find_report(raw: &str) -> String {
    binaryheap_iter_find_api::dead_binaryheap_iter_find_report(raw)
}
