pub fn selected_vec_retain_map_report(raw: &str) -> String {
    vec_retain_map_model::selected_vec_retain_map(raw)
}

pub fn dead_live_vec_retain_map_report(raw: &str) -> String {
    format!("dead-vec-retain-map-live-report:{raw}")
}
