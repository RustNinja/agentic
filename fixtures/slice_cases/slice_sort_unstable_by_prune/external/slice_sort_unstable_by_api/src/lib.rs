mod live;

pub use live::selected_slice_sort_unstable_by_report;

pub fn dead_slice_sort_unstable_by_report(raw: &str) -> String {
    format!("dead-slice-sort-unstable-by-report:{raw}")
}
