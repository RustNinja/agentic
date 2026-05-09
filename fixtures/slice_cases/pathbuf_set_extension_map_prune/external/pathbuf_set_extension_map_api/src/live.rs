pub fn selected_pathbuf_set_extension_map_report(raw: &str) -> String {
    pathbuf_set_extension_map_model::selected_pathbuf_set_extension_map(raw)
}

pub fn dead_live_pathbuf_set_extension_map_report(raw: &str) -> String {
    format!("dead-pathbuf-set-extension-map-live-report:{raw}")
}
