pub fn selected_iterator_inspect_map_report(raw: &str) -> String {
    iterator_inspect_map_model::selected_iterator_inspect_map(raw)
}

pub fn dead_live_iterator_inspect_map_report(raw: &str) -> String {
    format!("dead-live-iterator-inspect-map:{raw}")
}
