pub fn selected_hashset_replace_report(raw: &str) -> String {
    hashset_replace_model::selected_hashset_replace(raw)
}

pub fn dead_live_hashset_replace_report(raw: &str) -> String {
    format!("dead-hashset-replace-live-report:{raw}")
}
