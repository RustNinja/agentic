pub fn selected_pathbuf_with_extension_to_str_map_report(raw: &str) -> String {
    pathbuf_with_extension_to_str_map_model::selected_pathbuf_with_extension_to_str_map(raw)
}

pub fn dead_live_pathbuf_with_extension_to_str_map_report(raw: &str) -> String {
    format!("dead-pathbuf-with-extension-to-str-map-live-report:{raw}")
}
