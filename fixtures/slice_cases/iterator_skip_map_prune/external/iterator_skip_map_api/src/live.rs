pub fn selected_iterator_skip_map_report(raw: &str) -> String {
    iterator_skip_map_model::selected_iterator_skip_map(raw)
}

pub fn dead_live_iterator_skip_map_report(raw: &str) -> String {
    format!("dead-live-iterator-skip-map:{raw}")
}
