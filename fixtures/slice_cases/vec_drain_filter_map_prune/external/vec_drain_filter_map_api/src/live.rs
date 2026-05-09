pub fn selected_vec_drain_filter_map_report(raw: &str) -> String {
    vec_drain_filter_map_model::selected_vec_drain_filter_map(raw)
}

pub fn dead_live_vec_drain_filter_map_report(raw: &str) -> String {
    format!("dead-vec-drain-filter-map-live-report:{raw}")
}
