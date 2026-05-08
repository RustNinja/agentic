pub fn selected_deref_report(raw: &str) -> String {
    deref_model::selected_deref(raw)
}

pub fn dead_live_deref_report(raw: &str) -> String {
    format!("dead-live-deref:{raw}")
}
