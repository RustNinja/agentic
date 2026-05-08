pub fn selected_envelope_report(raw: &str) -> String {
    envelope_model::render_envelope(raw)
}

pub fn dead_live_envelope_report(raw: &str) -> String {
    format!("dead-live-envelope:{raw}")
}
