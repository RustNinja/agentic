pub fn selected_slice_last_chunk_mut_map_report(raw: &str) -> String {
    slice_last_chunk_mut_map_model::selected_slice_last_chunk_mut_map(raw)
}

pub fn dead_live_slice_last_chunk_mut_map_report(raw: &str) -> String {
    format!("dead-live-slice_last_chunk_mut_map-report:{raw}")
}
