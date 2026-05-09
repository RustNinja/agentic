pub fn selected_pathbuf_pop_map_report(raw: &str) -> String {
    pathbuf_pop_map_model::selected_pathbuf_pop_map(raw)
}

pub fn dead_live_pathbuf_pop_map_report(raw: &str) -> String {
    format!("dead-pathbuf-pop-map-live-report:{raw}")
}
