pub fn selected_vec_shrink_to_fit_last_map_report(raw: &str) -> String {
    vec_shrink_to_fit_last_map_model::selected_vec_shrink_to_fit_last_map(raw)
}

pub fn dead_live_vec_shrink_to_fit_last_map_report(raw: &str) -> String {
    format!("dead-vec-shrink-to-fit-last-map-live-report:{raw}")
}
