pub fn selected_dedup_by_report(raw: &str) -> String {
    dedup_model::selected_dedup_by(raw)
}

pub fn dead_live_dedup_by_report(raw: &str) -> String {
    format!("dead-live-dedup:{raw}")
}
