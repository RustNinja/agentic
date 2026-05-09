pub fn selected_hashset_get_report(raw: &str) -> String {
    hashset_get_model::selected_hashset_get(raw)
}

pub fn dead_live_hashset_get_report(raw: &str) -> String {
    format!("dead-hashset-get-live-report:{raw}")
}
