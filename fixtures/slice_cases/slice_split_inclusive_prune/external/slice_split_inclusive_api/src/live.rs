pub fn selected_slice_split_inclusive_report(raw: &str) -> String {
    slice_split_inclusive_model::selected_slice_split_inclusive(raw)
}

pub fn dead_live_slice_split_inclusive_report(raw: &str) -> String {
    format!("dead-slice-split-inclusive-live-report:{raw}")
}
