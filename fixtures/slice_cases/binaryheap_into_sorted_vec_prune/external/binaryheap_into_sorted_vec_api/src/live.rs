pub fn selected_binaryheap_into_sorted_vec_report(raw: &str) -> String {
    binaryheap_into_sorted_vec_model::selected_binaryheap_into_sorted_vec(raw)
}

pub fn dead_live_binaryheap_into_sorted_vec_report(raw: &str) -> String {
    format!("dead-binaryheap-into-sorted-vec-live-report:{raw}")
}
