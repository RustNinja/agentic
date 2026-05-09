mod live;

pub use live::selected_slice_select_nth_unstable_by_report;

pub fn dead_slice_select_nth_unstable_by_report(raw: &str) -> String {
    format!("dead-slice-select-nth-unstable-by-report:{raw}")
}
