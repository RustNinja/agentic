pub fn selected_binaryheap_append_into_sorted_vec_report(raw: &str) -> String {
    binaryheap_append_into_sorted_vec_model::selected_binaryheap_append_into_sorted_vec(raw)
}

pub fn dead_live_binaryheap_append_into_sorted_vec_report(raw: &str) -> String {
    format!("dead-live-binaryheap-append-into-sorted-vec:{raw}")
}
