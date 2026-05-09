use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_pop_report(raw: &str) -> String {
    binaryheap_pop_api::selected_binaryheap_pop_report(raw)
}

pub fn dead_binaryheap_pop_report(raw: &str) -> String {
    binaryheap_pop_api::dead_binaryheap_pop_report(raw)
}
