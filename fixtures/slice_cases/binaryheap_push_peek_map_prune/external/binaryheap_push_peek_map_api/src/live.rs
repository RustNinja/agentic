pub fn selected_binaryheap_push_peek_map_report(raw: &str) -> String {
    binaryheap_push_peek_map_model::selected_binaryheap_push_peek_map(raw)
}

pub fn dead_live_binaryheap_push_peek_map_report(raw: &str) -> String {
    format!("dead-binaryheap-push-peek-map-live-report:{raw}")
}
