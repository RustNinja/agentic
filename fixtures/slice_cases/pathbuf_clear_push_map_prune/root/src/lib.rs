use opensourced::opensourced;

#[opensourced]
pub fn selected_pathbuf_clear_push_map_report(raw: &str) -> String {
    pathbuf_clear_push_map_api::selected_pathbuf_clear_push_map_report(raw)
}

pub fn dead_pathbuf_clear_push_map_report(raw: &str) -> String {
    pathbuf_clear_push_map_api::dead_pathbuf_clear_push_map_report(raw)
}
