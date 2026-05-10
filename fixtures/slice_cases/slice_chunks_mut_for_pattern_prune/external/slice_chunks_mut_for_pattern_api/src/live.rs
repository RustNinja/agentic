pub fn selected_slice_chunks_mut_for_pattern_report(raw: &str) -> String {
    slice_chunks_mut_for_pattern_model::selected_slice_chunks_mut_for_pattern(raw)
}

pub fn dead_live_slice_chunks_mut_for_pattern_report(raw: &str) -> String {
    format!("dead-live-slice-chunks-mut-for-pattern-report:{raw}")
}
