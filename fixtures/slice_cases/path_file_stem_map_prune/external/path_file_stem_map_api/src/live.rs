pub fn selected_path_file_stem_map_report(raw: &str) -> String {
    path_file_stem_map_model::selected_path_file_stem_map(raw)
}

pub fn dead_live_path_file_stem_map_report(raw: &str) -> String {
    format!("dead-path-file-stem-map-live-report:{raw}")
}
