pub fn selected_vec_split_at_map_report(raw: &str) -> String {
    vec_split_at_map_model::selected_vec_split_at_map(raw)
}

pub fn dead_live_vec_split_at_map_report(raw: &str) -> String {
    format!("dead-vec-split-at-map-live-report:{raw}")
}
