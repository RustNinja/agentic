use opensourced::opensourced;

#[opensourced]
pub fn selected_pathbuf_push_to_str_map_report(raw: &str) -> String {
    pathbuf_push_to_str_map_api::selected_pathbuf_push_to_str_map_report(raw)
}

pub fn dead_pathbuf_push_to_str_map_report(raw: &str) -> String {
    pathbuf_push_to_str_map_api::dead_pathbuf_push_to_str_map_report(raw)
}
