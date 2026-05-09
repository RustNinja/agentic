mod live;

pub use live::selected_slice_split_report;

pub fn dead_slice_split_report(raw: &str) -> String {
    format!("dead-slice-split-report:{raw}")
}
