pub fn selected_protocol_report(raw: &str) -> String {
    protocol_model::selected_protocol(raw)
}

pub fn dead_live_protocol_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
