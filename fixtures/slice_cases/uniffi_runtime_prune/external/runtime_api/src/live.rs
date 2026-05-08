pub fn selected_runtime_report(raw: &str) -> String {
    runtime_support::selected_runtime(raw)
}

pub fn dead_live_runtime_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
