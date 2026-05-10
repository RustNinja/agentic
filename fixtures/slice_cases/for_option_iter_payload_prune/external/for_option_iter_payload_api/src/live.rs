pub fn selected_for_option_iter_payload_report(raw: &str) -> String {
    for_option_iter_payload_model::selected_for_option_iter_payload(raw)
}

pub fn dead_live_for_option_iter_payload_report(raw: &str) -> String {
    format!("dead-live-for-option-iter-payload-report:{raw}")
}
