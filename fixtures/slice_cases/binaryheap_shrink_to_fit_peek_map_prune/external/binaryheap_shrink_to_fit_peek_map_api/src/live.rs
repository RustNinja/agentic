pub fn selected_binaryheap_shrink_to_fit_peek_map_report(raw: &str) -> String {
    binaryheap_shrink_to_fit_peek_map_model::selected_binaryheap_shrink_to_fit_peek_map(raw)
}

pub fn dead_live_binaryheap_shrink_to_fit_peek_map_report(raw: &str) -> String {
    format!("dead-binaryheap-shrink-to-fit-peek-map-live-report:{raw}")
}
