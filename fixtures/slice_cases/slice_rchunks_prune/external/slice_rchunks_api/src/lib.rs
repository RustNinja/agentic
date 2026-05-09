mod live;

pub use live::selected_slice_rchunks_report;

pub fn dead_slice_rchunks_report(raw: &str) -> String {
    format!("dead-slice-rchunks-report:{raw}")
}
