pub fn selected_slice_as_chunks_iter_report(raw: &str) -> String {
    slice_as_chunks_iter_model::selected_slice_as_chunks_iter(raw)
}

pub fn dead_live_slice_as_chunks_iter_report(raw: &str) -> String {
    format!("dead-live-slice_as_chunks_iter-report:{raw}")
}
