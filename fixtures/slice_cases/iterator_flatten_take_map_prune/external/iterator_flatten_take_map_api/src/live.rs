pub fn selected_iterator_flatten_take_map_report(raw: &str) -> String {
    iterator_flatten_take_map_model::selected_iterator_flatten_take_map(raw)
}

pub fn dead_live_iterator_flatten_take_map_report(raw: &str) -> String {
    format!("dead-iterator-flatten-take-map-live-report:{raw}")
}
