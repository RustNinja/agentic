mod live;

pub use live::selected_vec_sort_by_cached_key_report;

pub fn dead_vec_sort_by_cached_key_report(raw: &str) -> String {
    format!("dead-vec-sort-by-cached-key-report:{raw}")
}
