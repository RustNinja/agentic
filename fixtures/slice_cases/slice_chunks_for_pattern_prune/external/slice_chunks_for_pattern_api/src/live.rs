pub fn selected_slice_chunks_for_pattern_report(raw: &str) -> String {
    slice_chunks_for_pattern_model::selected_slice_chunks_for_pattern(raw)
}

pub fn dead_live_slice_chunks_for_pattern_report(raw: &str) -> String {
    format!("dead-live-slice-chunks-for-pattern-report:{raw}")
}
