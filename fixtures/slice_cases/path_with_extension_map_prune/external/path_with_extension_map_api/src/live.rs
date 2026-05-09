pub fn selected_path_with_extension_map_report(raw: &str) -> String {
    path_with_extension_map_model::selected_path_with_extension_map(raw)
}

pub fn dead_live_path_with_extension_map_report(raw: &str) -> String {
    format!("dead-path-with-extension-map-live-report:{raw}")
}
