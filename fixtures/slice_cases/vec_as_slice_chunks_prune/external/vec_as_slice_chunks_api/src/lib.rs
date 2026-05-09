mod live;

pub use live::selected_vec_as_slice_chunks_report;

pub fn dead_vec_as_slice_chunks_report(raw: &str) -> String {
    format!("dead-vec-as-slice-chunks-report:{raw}")
}
