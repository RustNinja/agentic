pub fn selected_guard_report(raw: &str) -> String {
    guard_model::selected_guard(raw)
}

pub fn dead_live_guard_report(raw: &str) -> String {
    format!("dead-live-guard:{raw}")
}
