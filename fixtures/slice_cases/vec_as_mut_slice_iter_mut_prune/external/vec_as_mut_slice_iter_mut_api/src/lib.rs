mod live;

pub use live::selected_vec_as_mut_slice_iter_mut_report;

pub fn dead_vec_as_mut_slice_iter_mut_report(raw: &str) -> String {
    format!("dead-vec-as-mut-slice-iter-mut-report:{raw}")
}
