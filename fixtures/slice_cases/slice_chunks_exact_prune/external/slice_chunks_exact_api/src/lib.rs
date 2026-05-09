mod live;

pub use live::selected_slice_chunks_exact_report;

pub fn dead_slice_chunks_exact_report(raw: &str) -> String {
    format!("dead-slice-chunks-exact-report:{raw}")
}
