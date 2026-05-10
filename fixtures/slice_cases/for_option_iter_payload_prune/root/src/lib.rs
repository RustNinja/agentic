use opensourced::opensourced;

#[opensourced]
pub fn selected_for_option_iter_payload_report(raw: &str) -> String {
    for_option_iter_payload_api::selected_for_option_iter_payload_report(raw)
}

pub fn dead_for_option_iter_payload_report(raw: &str) -> String {
    for_option_iter_payload_api::dead_for_option_iter_payload_report(raw)
}
