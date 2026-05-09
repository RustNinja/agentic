pub fn selected_iterator_peekable_peek_map_report(raw: &str) -> String {
    iterator_peekable_peek_map_model::selected_iterator_peekable_peek_map(raw)
}

pub fn dead_live_iterator_peekable_peek_map_report(raw: &str) -> String {
    format!("dead-iterator-peekable-peek-map-live-report:{raw}")
}
