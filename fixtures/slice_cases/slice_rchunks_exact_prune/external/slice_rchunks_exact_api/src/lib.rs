mod live;

pub use live::selected_slice_rchunks_exact_report;

pub fn dead_slice_rchunks_exact_report(raw: &str) -> String {
    format!("dead-slice-rchunks-exact-report:{raw}")
}
