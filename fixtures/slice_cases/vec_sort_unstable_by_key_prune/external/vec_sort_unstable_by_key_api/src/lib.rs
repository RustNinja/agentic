mod live;

pub use live::selected_vec_sort_unstable_by_key_report;

pub fn dead_vec_sort_unstable_by_key_report(raw: &str) -> String {
    format!("dead-vec-sort-unstable-by-key-report:{raw}")
}
