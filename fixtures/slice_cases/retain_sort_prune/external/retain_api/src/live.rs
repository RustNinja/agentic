pub fn selected_retain_report(raw: &str) -> String {
    retain_model::selected_retain(raw)
}

pub fn dead_live_retain_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
