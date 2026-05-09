pub fn selected_vec_splice_map_report(raw: &str) -> String {
    vec_splice_map_model::selected_vec_splice_map(raw)
}

pub fn dead_live_vec_splice_map_report(raw: &str) -> String {
    format!("dead-live-vec-splice-map:{raw}")
}
