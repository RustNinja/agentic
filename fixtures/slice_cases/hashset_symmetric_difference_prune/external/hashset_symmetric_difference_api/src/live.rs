pub fn selected_hashset_symmetric_difference_report(raw: &str) -> String {
    hashset_symmetric_difference_model::selected_hashset_symmetric_difference(raw)
}

pub fn dead_live_hashset_symmetric_difference_report(raw: &str) -> String {
    format!("dead-live-hashset-symmetric-difference:{raw}")
}
