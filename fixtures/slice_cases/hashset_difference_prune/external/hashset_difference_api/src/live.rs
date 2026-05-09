pub fn selected_hashset_difference_report(raw: &str) -> String {
    hashset_difference_model::selected_hashset_difference(raw)
}

pub fn dead_live_hashset_difference_report(raw: &str) -> String {
    format!("dead-live-hashset-difference:{raw}")
}
