use opensourced::opensourced;

#[opensourced]
pub fn selected_binaryheap_into_sorted_vec_report(raw: &str) -> String {
    binaryheap_into_sorted_vec_api::selected_binaryheap_into_sorted_vec_report(raw)
}

pub fn dead_binaryheap_into_sorted_vec_report(raw: &str) -> String {
    binaryheap_into_sorted_vec_api::dead_binaryheap_into_sorted_vec_report(raw)
}
