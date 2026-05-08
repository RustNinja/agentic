pub fn selected_generic_report(raw: &str) -> String {
    generic_model::selected_generic(raw)
}

pub fn dead_live_generic_report(raw: &str) -> String {
    format!("dead-live-generic:{raw}")
}
