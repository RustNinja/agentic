use opensourced::opensourced;

#[opensourced]
pub fn selected_envelope_report(raw: &str) -> String {
    envelope_api::selected_envelope_report(raw)
}

pub fn dead_envelope_report(raw: &str) -> String {
    envelope_api::dead_envelope_report(raw)
}
