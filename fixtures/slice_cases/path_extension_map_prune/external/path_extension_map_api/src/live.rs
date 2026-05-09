pub fn selected_path_extension_map_report(raw: &str) -> String {
    path_extension_map_model::selected_path_extension_map(raw)
}

pub fn dead_live_path_extension_map_report(raw: &str) -> String {
    format!("dead-path-extension-map-live-report:{raw}")
}
