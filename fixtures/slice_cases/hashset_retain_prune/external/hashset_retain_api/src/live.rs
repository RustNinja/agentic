pub fn selected_hashset_retain_report(raw: &str) -> String {
    hashset_retain_model::selected_hashset_retain(raw)
}

pub fn dead_live_hashset_retain_report(raw: &str) -> String {
    format!("dead-hashset-retain-live-report:{raw}")
}
