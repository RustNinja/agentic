pub fn selected_collect_hashset_report(raw: &str) -> String {
    collect_hashset_model::selected_collect_hashset(raw)
}

pub fn dead_live_collect_hashset_report(raw: &str) -> String {
    format!("dead-collect-hashset-live-report:{raw}")
}
