pub fn selected_slice_split_report(raw: &str) -> String {
    slice_split_model::selected_slice_split(raw)
}

pub fn dead_live_slice_split_report(raw: &str) -> String {
    format!("dead-slice-split-live-report:{raw}")
}
