pub fn selected_iter_once_map_report(raw: &str) -> String {
    iter_once_map_model::selected_iter_once_map(raw)
}

pub fn dead_live_iter_once_map_report(raw: &str) -> String {
    format!("dead-live-iter-once-map-report:{raw}")
}
