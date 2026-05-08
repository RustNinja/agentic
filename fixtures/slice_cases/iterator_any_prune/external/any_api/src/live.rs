pub fn selected_any_report(raw: &str) -> String {
    any_model::selected_any(raw)
}

pub fn dead_live_any_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
