mod live;

pub use live::selected_slice_split_inclusive_report;

pub fn dead_slice_split_inclusive_report(raw: &str) -> String {
    format!("dead-slice-split-inclusive-report:{raw}")
}
