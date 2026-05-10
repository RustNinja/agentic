pub fn selected_vec_sort_by_map_report(raw: &str) -> String {
    vec_sort_by_map_model::selected_vec_sort_by_map(raw)
}

pub fn dead_live_vec_sort_by_map_report(raw: &str) -> String {
    format!("dead-vec-sort-by-map-live-report:{raw}")
}
