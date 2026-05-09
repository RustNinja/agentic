pub fn selected_hashset_drain_report(raw: &str) -> String {
    hashset_drain_model::selected_hashset_drain(raw)
}

pub fn dead_live_hashset_drain_report(raw: &str) -> String {
    format!("dead-hashset-drain-live-report:{raw}")
}
