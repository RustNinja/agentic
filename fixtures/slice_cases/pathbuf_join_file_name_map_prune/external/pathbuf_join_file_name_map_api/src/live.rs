pub fn selected_pathbuf_join_file_name_map_report(raw: &str) -> String {
    pathbuf_join_file_name_map_model::selected_pathbuf_join_file_name_map(raw)
}

pub fn dead_live_pathbuf_join_file_name_map_report(raw: &str) -> String {
    format!("dead-pathbuf-join-file-name-map-live-report:{raw}")
}
