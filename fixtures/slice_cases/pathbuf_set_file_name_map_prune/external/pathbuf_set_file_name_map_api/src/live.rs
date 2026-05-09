pub fn selected_pathbuf_set_file_name_map_report(raw: &str) -> String {
    pathbuf_set_file_name_map_model::selected_pathbuf_set_file_name_map(raw)
}

pub fn dead_live_pathbuf_set_file_name_map_report(raw: &str) -> String {
    format!("dead-pathbuf-set-file-name-map-live-report:{raw}")
}
