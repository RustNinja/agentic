mod live;

pub use live::selected_slice_iter_mut_report;

pub fn dead_slice_iter_mut_report(raw: &str) -> String {
    format!("dead-slice-iter-mut-report:{raw}")
}
