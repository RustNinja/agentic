pub fn selected_slice_sort_unstable_by_report(raw: &str) -> String {
    slice_sort_unstable_by_model::selected_slice_sort_unstable_by(raw)
}

pub fn dead_live_slice_sort_unstable_by_report(raw: &str) -> String {
    format!("dead-slice-sort-unstable-by-live-report:{raw}")
}
