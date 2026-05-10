pub fn selected_binaryheap_append_peek_map_report(raw: &str) -> String {
    binaryheap_append_peek_map_model::selected_binaryheap_append_peek_map(raw)
}

pub fn dead_live_binaryheap_append_peek_map_report(raw: &str) -> String {
    format!("dead-binaryheap-append-peek-map-live-report:{raw}")
}
