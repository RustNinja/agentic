pub fn selected_slice_rchunks_exact_report(raw: &str) -> String {
    slice_rchunks_exact_model::selected_slice_rchunks_exact(raw)
}

pub fn dead_live_slice_rchunks_exact_report(raw: &str) -> String {
    format!("dead-slice-rchunks-exact-live-report:{raw}")
}
