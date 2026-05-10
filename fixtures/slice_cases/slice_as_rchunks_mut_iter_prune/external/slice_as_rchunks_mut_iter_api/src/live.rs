pub fn selected_slice_as_rchunks_mut_iter_report(raw: &str) -> String {
    slice_as_rchunks_mut_iter_model::selected_slice_as_rchunks_mut_iter(raw)
}

pub fn dead_live_slice_as_rchunks_mut_iter_report(raw: &str) -> String {
    format!("dead-live-slice_as_rchunks_mut_iter-report:{raw}")
}
