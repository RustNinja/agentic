use opensourced::opensourced;

#[opensourced]
pub fn selected_protocol_report(raw: &str) -> String {
    protocol_api::selected_protocol_report(raw)
}

pub fn dead_protocol_report(raw: &str) -> String {
    protocol_api::dead_protocol_report(raw)
}
