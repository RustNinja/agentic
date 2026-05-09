mod live;

pub use live::selected_slice_chunks_report;

pub fn dead_slice_chunks_report(raw: &str) -> String {
    format!("dead-slice-chunks-report:{raw}")
}
