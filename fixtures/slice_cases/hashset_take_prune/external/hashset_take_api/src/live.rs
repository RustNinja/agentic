pub fn selected_hashset_take_report(raw: &str) -> String {
    hashset_take_model::selected_hashset_take(raw)
}

pub fn dead_live_hashset_take_report(raw: &str) -> String {
    format!("dead-hashset-take-live-report:{raw}")
}
