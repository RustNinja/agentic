pub fn selected_pathbuf_reserve_push_map_report(raw: &str) -> String {
    pathbuf_reserve_push_map_model::selected_pathbuf_reserve_push_map(raw)
}

pub fn dead_live_pathbuf_reserve_push_map_report(raw: &str) -> String {
    format!("dead-pathbuf-reserve-push-map-live-report:{raw}")
}
