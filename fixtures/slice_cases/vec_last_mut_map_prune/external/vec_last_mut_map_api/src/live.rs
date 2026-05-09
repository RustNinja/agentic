pub fn selected_vec_last_mut_map_report(raw: &str) -> String {
    vec_last_mut_map_model::selected_vec_last_mut_map(raw)
}

pub fn dead_live_vec_last_mut_map_report(raw: &str) -> String {
    format!("dead-vec-last-mut-map-live-report:{raw}")
}
