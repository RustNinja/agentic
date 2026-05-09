pub fn selected_vec_first_mut_map_report(raw: &str) -> String {
    vec_first_mut_map_model::selected_vec_first_mut_map(raw)
}

pub fn dead_live_vec_first_mut_map_report(raw: &str) -> String {
    format!("dead-vec-first-mut-map-live-report:{raw}")
}
