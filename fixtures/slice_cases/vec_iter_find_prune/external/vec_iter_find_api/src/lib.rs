mod live;

pub use live::selected_vec_iter_find_report;

pub fn dead_vec_iter_find_report(raw: &str) -> String {
    format!("dead-vec-iter-find-report:{raw}")
}
