mod live;

pub use live::selected_slice_rsplit_report;

pub fn dead_slice_rsplit_report(raw: &str) -> String {
    format!("dead-slice-rsplit-report:{raw}")
}
