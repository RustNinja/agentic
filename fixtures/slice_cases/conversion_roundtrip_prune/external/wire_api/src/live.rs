pub fn selected_wire_report(raw: &str) -> String {
    wire_model::selected_wire(raw)
}

pub fn dead_live_wire_report(raw: &str) -> String {
    format!("dead-live-wire:{raw}")
}
