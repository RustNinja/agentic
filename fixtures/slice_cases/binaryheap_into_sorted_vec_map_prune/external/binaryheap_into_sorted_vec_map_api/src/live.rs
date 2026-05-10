pub fn selected_binaryheap_into_sorted_vec_map_report(raw: &str) -> String {
    binaryheap_into_sorted_vec_map_model::selected_binaryheap_into_sorted_vec_map(raw)
}

pub fn dead_live_binaryheap_into_sorted_vec_map_report(raw: &str) -> String {
    format!("dead-binaryheap-into-sorted-vec-map-live-report:{raw}")
}
