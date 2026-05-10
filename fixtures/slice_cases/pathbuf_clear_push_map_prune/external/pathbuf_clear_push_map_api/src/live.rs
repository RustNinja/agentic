pub fn selected_pathbuf_clear_push_map_report(raw: &str) -> String {
    pathbuf_clear_push_map_model::selected_pathbuf_clear_push_map(raw)
}

pub fn dead_live_pathbuf_clear_push_map_report(raw: &str) -> String {
    format!("dead-pathbuf-clear-push-map-live-report:{raw}")
}
