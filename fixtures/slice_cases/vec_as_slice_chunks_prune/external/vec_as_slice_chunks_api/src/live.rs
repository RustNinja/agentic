pub fn selected_vec_as_slice_chunks_report(raw: &str) -> String {
    vec_as_slice_chunks_model::selected_vec_as_slice_chunks(raw)
}

pub fn dead_live_vec_as_slice_chunks_report(raw: &str) -> String {
    format!("dead-vec-as-slice-chunks-live-report:{raw}")
}
