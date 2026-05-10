pub fn selected_iterator_peekable_peek_mut_report(raw: &str) -> String {
    iterator_peekable_peek_mut_model::selected_iterator_peekable_peek_mut(raw)
}

pub fn dead_live_iterator_peekable_peek_mut_report(raw: &str) -> String {
    format!("dead-live-iterator-peekable-peek-mut-report:{raw}")
}
