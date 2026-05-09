pub fn selected_path_parent_map_report(raw: &str) -> String {
    path_parent_map_model::selected_path_parent_map(raw)
}

pub fn dead_live_path_parent_map_report(raw: &str) -> String {
    format!("dead-path-parent-map-live-report:{raw}")
}
