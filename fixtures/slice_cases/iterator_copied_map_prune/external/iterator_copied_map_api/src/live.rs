pub fn selected_iterator_copied_map_report(raw: &str) -> String {
    iterator_copied_map_model::selected_iterator_copied_map(raw)
}

pub fn dead_live_iterator_copied_map_report(raw: &str) -> String {
    format!("dead-live-iterator-copied-map:{raw}")
}
