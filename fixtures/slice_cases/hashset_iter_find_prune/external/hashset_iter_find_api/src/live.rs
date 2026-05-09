pub fn selected_hashset_iter_find_report(raw: &str) -> String {
    hashset_iter_find_model::selected_hashset_iter_find(raw)
}

pub fn dead_live_hashset_iter_find_report(raw: &str) -> String {
    format!("dead-hashset-iter-find-live-report:{raw}")
}
