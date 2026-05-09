pub fn selected_slice_chunks_exact_report(raw: &str) -> String {
    slice_chunks_exact_model::selected_slice_chunks_exact(raw)
}

pub fn dead_live_slice_chunks_exact_report(raw: &str) -> String {
    format!("dead-slice-chunks-exact-live-report:{raw}")
}
