pub fn selected_path_ancestors_find_map_report(raw: &str) -> String {
    path_ancestors_find_map_model::selected_path_ancestors_find_map(raw)
}

pub fn dead_live_path_ancestors_find_map_report(raw: &str) -> String {
    format!("dead-path-ancestors-find-map-live-report:{raw}")
}
