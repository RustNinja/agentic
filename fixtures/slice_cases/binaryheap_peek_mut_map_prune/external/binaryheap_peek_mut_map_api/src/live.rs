pub fn selected_binaryheap_peek_mut_map_report(raw: &str) -> String {
    binaryheap_peek_mut_map_model::selected_binaryheap_peek_mut_map(raw)
}

pub fn dead_live_binaryheap_peek_mut_map_report(raw: &str) -> String {
    format!("dead-binaryheap-peek-mut-map-live-report:{raw}")
}
