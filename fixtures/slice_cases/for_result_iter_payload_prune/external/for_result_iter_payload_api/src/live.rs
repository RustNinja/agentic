pub fn selected_for_result_iter_payload_report(raw: &str) -> String {
    for_result_iter_payload_model::selected_for_result_iter_payload(raw)
}

pub fn dead_live_for_result_iter_payload_report(raw: &str) -> String {
    format!("dead-live-for-result-iter-payload-report:{raw}")
}
