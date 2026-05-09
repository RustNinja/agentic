use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_binaryheap_report(raw: &str) -> String {
    collect_binaryheap_api::selected_collect_binaryheap_report(raw)
}

pub fn dead_collect_binaryheap_report(raw: &str) -> String {
    collect_binaryheap_api::dead_collect_binaryheap_report(raw)
}
