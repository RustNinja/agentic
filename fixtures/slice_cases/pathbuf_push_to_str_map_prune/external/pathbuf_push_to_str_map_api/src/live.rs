pub fn selected_pathbuf_push_to_str_map_report(raw: &str) -> String {
    pathbuf_push_to_str_map_model::selected_pathbuf_push_to_str_map(raw)
}

pub fn dead_live_pathbuf_push_to_str_map_report(raw: &str) -> String {
    format!("dead-pathbuf-push-to-str-map-live-report:{raw}")
}
