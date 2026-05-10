pub fn selected_vec_chunks_exact_map_report(raw: &str) -> String {
    vec_chunks_exact_map_model::selected_vec_chunks_exact_map(raw)
}

pub fn dead_live_vec_chunks_exact_map_report(raw: &str) -> String {
    format!("dead-vec-chunks-exact-map-live-report:{raw}")
}
