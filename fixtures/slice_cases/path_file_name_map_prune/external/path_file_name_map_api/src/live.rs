pub fn selected_path_file_name_map_report(raw: &str) -> String {
    path_file_name_map_model::selected_path_file_name_map(raw)
}

pub fn dead_live_path_file_name_map_report(raw: &str) -> String {
    format!("dead-path-file-name-map-live-report:{raw}")
}
