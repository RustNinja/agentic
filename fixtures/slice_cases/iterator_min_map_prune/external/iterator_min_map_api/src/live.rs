pub fn selected_iterator_min_map_report(raw: &str) -> String {
    iterator_min_map_model::selected_iterator_min_map(raw)
}

pub fn dead_live_iterator_min_map_report(raw: &str) -> String {
    format!("dead-live-iterator-min-map-report:{raw}")
}
