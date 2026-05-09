pub fn selected_slice_rchunks_report(raw: &str) -> String {
    slice_rchunks_model::selected_slice_rchunks(raw)
}

pub fn dead_live_slice_rchunks_report(raw: &str) -> String {
    format!("dead-slice-rchunks-live-report:{raw}")
}
