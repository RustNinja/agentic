pub fn selected_iterator_filter_predicate_map_report(raw: &str) -> String {
    iterator_filter_predicate_map_model::selected_iterator_filter_predicate_map(raw)
}

pub fn dead_live_iterator_filter_predicate_map_report(raw: &str) -> String {
    format!("dead-live-iterator-filter-predicate-map:{raw}")
}
