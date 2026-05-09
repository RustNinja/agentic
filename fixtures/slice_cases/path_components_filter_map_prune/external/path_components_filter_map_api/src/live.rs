pub fn selected_path_components_filter_map_report(raw: &str) -> String {
    path_components_filter_map_model::selected_path_components_filter_map(raw)
}

pub fn dead_live_path_components_filter_map_report(raw: &str) -> String {
    format!("dead-path-components-filter-map-live-report:{raw}")
}
