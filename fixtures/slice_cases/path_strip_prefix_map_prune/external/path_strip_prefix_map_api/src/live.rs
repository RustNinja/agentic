pub fn selected_path_strip_prefix_map_report(raw: &str) -> String {
    path_strip_prefix_map_model::selected_path_strip_prefix_map(raw)
}

pub fn dead_live_path_strip_prefix_map_report(raw: &str) -> String {
    format!("dead-path-strip-prefix-map-live-report:{raw}")
}
