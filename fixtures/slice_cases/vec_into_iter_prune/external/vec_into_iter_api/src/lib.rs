mod live;

pub use live::selected_vec_into_iter_report;

pub fn dead_vec_into_iter_report(raw: &str) -> String {
    format!("dead-vec-into-iter-report:{raw}")
}
