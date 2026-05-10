pub fn selected_vec_extend_from_within_map_report(raw: &str) -> String {
    vec_extend_from_within_map_model::selected_vec_extend_from_within_map(raw)
}

pub fn dead_live_vec_extend_from_within_map_report(raw: &str) -> String {
    format!("dead-vec-extend-from-within-map-live-report:{raw}")
}
