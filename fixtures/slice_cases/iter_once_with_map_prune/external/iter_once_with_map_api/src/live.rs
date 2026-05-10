pub fn selected_iter_once_with_map_report(raw: &str) -> String {
    iter_once_with_map_model::selected_iter_once_with_map(raw)
}

pub fn dead_live_iter_once_with_map_report(raw: &str) -> String {
    format!("dead-live-iter-once-with-map-report:{raw}")
}
