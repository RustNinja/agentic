pub fn selected_binaryheap_from_iter_peek_map_report(raw: &str) -> String {
    binaryheap_from_iter_peek_map_model::selected_binaryheap_from_iter_peek_map(raw)
}

pub fn dead_live_binaryheap_from_iter_peek_map_report(raw: &str) -> String {
    format!("dead-binaryheap-from-iter-peek-map-live-report:{raw}")
}
