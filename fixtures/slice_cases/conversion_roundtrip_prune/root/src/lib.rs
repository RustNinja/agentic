use opensourced::opensourced;

#[opensourced]
pub fn selected_wire_report(raw: &str) -> String {
    wire_api::selected_wire_report(raw)
}

pub fn dead_wire_report(raw: &str) -> String {
    wire_api::dead_wire_report(raw)
}
