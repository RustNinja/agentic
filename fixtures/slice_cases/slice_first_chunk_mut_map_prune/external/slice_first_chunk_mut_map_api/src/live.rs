pub fn selected_slice_first_chunk_mut_map_report(raw: &str) -> String {
    slice_first_chunk_mut_map_model::selected_slice_first_chunk_mut_map(raw)
}

pub fn dead_live_slice_first_chunk_mut_map_report(raw: &str) -> String {
    format!("dead-live-slice_first_chunk_mut_map-report:{raw}")
}
