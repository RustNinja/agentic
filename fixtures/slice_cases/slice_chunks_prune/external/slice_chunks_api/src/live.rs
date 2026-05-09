pub fn selected_slice_chunks_report(raw: &str) -> String {
    slice_chunks_model::selected_slice_chunks(raw)
}

pub fn dead_live_slice_chunks_report(raw: &str) -> String {
    format!("dead-slice-chunks-live-report:{raw}")
}
