pub fn selected_for_hashset_ref_payload_report(raw: &str) -> String {
    for_hashset_ref_payload_model::selected_for_hashset_ref_payload(raw)
}

pub fn dead_live_for_hashset_ref_payload_report(raw: &str) -> String {
    format!("dead-live-for-hashset-ref-payload-report:{raw}")
}
