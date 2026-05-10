pub fn selected_for_vec_mut_payload_report(raw: &str) -> String {
    for_vec_mut_payload_model::selected_for_vec_mut_payload(raw)
}

pub fn dead_live_for_vec_mut_payload_report(raw: &str) -> String {
    format!("dead-live-for-vec-mut-payload-report:{raw}")
}
