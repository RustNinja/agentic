pub fn selected_slice_rsplit_report(raw: &str) -> String {
    slice_rsplit_model::selected_slice_rsplit(raw)
}

pub fn dead_live_slice_rsplit_report(raw: &str) -> String {
    format!("dead-slice-rsplit-live-report:{raw}")
}
