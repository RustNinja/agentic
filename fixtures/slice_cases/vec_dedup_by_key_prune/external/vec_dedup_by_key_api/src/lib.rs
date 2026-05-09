mod live;

pub use live::selected_vec_dedup_by_key_report;

pub fn dead_vec_dedup_by_key_report(raw: &str) -> String {
    format!("dead-vec-dedup-by-key-report:{raw}")
}
