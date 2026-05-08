pub fn selected_registry_report(raw: &str) -> String {
    registry_support::selected_registry(raw)
}

pub fn dead_live_registry_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
