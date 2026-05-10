pub fn selected_slice_rchunks_mut_for_pattern_report(raw: &str) -> String {
    slice_rchunks_mut_for_pattern_model::selected_slice_rchunks_mut_for_pattern(raw)
}

pub fn dead_live_slice_rchunks_mut_for_pattern_report(raw: &str) -> String {
    format!("dead-live-slice-rchunks-mut-for-pattern-report:{raw}")
}
