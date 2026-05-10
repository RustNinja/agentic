pub fn selected_vec_dedup_by_map_report(raw: &str) -> String {
    vec_dedup_by_map_model::selected_vec_dedup_by_map(raw)
}

pub fn dead_live_vec_dedup_by_map_report(raw: &str) -> String {
    format!("dead-vec-dedup-by-map-live-report:{raw}")
}
