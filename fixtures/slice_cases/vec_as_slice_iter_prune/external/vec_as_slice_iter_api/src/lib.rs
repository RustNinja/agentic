mod live;

pub use live::selected_vec_as_slice_iter_report;

pub fn dead_vec_as_slice_iter_report(raw: &str) -> String {
    format!("dead-vec-as-slice-iter-report:{raw}")
}
