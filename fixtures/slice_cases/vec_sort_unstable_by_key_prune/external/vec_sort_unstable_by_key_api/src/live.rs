pub fn selected_vec_sort_unstable_by_key_report(raw: &str) -> String {
    vec_sort_unstable_by_key_model::selected_vec_sort_unstable_by_key(raw)
}

pub fn dead_live_vec_sort_unstable_by_key_report(raw: &str) -> String {
    format!("dead-vec-sort-unstable-by-key-live-report:{raw}")
}
