use opensourced::opensourced;

#[opensourced]
pub fn selected_pathbuf_set_extension_map_report(raw: &str) -> String {
    pathbuf_set_extension_map_api::selected_pathbuf_set_extension_map_report(raw)
}

pub fn dead_pathbuf_set_extension_map_report(raw: &str) -> String {
    pathbuf_set_extension_map_api::dead_pathbuf_set_extension_map_report(raw)
}
