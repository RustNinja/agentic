pub fn selected_hashset_into_iter_report(raw: &str) -> String {
    hashset_into_iter_model::selected_hashset_into_iter(raw)
}

pub fn dead_live_hashset_into_iter_report(raw: &str) -> String {
    format!("dead-hashset-into-iter-live-report:{raw}")
}
