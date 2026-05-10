pub fn selected_for_linkedlist_ref_payload_report(raw: &str) -> String {
    for_linkedlist_ref_payload_model::selected_for_linkedlist_ref_payload(raw)
}

pub fn dead_live_for_linkedlist_ref_payload_report(raw: &str) -> String {
    format!("dead-live-for-linkedlist-ref-payload-report:{raw}")
}
