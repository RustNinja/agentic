pub fn selected_binaryheap_push_pop_map_report(raw: &str) -> String {
    binaryheap_push_pop_map_model::selected_binaryheap_push_pop_map(raw)
}

pub fn dead_live_binaryheap_push_pop_map_report(raw: &str) -> String {
    format!("dead-binaryheap-push-pop-map-live-report:{raw}")
}
