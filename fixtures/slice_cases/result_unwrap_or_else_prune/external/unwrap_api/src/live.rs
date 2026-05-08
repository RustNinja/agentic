pub fn selected_unwrap_report(raw: &str) -> String {
    unwrap_model::selected_unwrap(raw)
}

pub fn dead_live_unwrap_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
