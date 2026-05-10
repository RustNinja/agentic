pub fn selected_vec_clear_extend_map_report(raw: &str) -> String {
    vec_clear_extend_map_model::selected_vec_clear_extend_map(raw)
}

pub fn dead_live_vec_clear_extend_map_report(raw: &str) -> String {
    format!("dead-vec-clear-extend-map-live-report:{raw}")
}
