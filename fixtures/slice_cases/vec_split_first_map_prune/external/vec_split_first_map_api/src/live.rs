pub fn selected_vec_split_first_map_report(raw: &str) -> String {
    vec_split_first_map_model::selected_vec_split_first_map(raw)
}

pub fn dead_live_vec_split_first_map_report(raw: &str) -> String {
    format!("dead-live-vec-split-first-map:{raw}")
}
