use opensourced::opensourced;

#[opensourced]
pub fn selected_pathbuf_join_file_name_map_report(raw: &str) -> String {
    pathbuf_join_file_name_map_api::selected_pathbuf_join_file_name_map_report(raw)
}

pub fn dead_pathbuf_join_file_name_map_report(raw: &str) -> String {
    pathbuf_join_file_name_map_api::dead_pathbuf_join_file_name_map_report(raw)
}
