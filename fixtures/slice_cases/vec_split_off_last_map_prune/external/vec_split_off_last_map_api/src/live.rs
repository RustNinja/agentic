pub fn selected_vec_split_off_last_map_report(raw: &str) -> String {
    vec_split_off_last_map_model::selected_vec_split_off_last_map(raw)
}

pub fn dead_live_vec_split_off_last_map_report(raw: &str) -> String {
    format!("dead-vec-split-off-last-map-live-report:{raw}")
}
