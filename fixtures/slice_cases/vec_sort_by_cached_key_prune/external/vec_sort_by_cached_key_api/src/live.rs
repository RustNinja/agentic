pub fn selected_vec_sort_by_cached_key_report(raw: &str) -> String {
    vec_sort_by_cached_key_model::selected_vec_sort_by_cached_key(raw)
}

pub fn dead_live_vec_sort_by_cached_key_report(raw: &str) -> String {
    format!("dead-vec-sort-by-cached-key-live-report:{raw}")
}
