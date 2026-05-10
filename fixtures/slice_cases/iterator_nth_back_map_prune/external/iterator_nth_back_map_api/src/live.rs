pub fn selected_iterator_nth_back_map_report(raw: &str) -> String {
    iterator_nth_back_map_model::selected_iterator_nth_back_map(raw)
}

pub fn dead_live_iterator_nth_back_map_report(raw: &str) -> String {
    format!("dead-live-iterator-nth-back-map-report:{raw}")
}
