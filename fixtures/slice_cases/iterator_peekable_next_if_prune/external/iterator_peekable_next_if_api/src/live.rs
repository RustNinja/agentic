pub fn selected_iterator_peekable_next_if_report(raw: &str) -> String {
    iterator_peekable_next_if_model::selected_iterator_peekable_next_if(raw)
}

pub fn dead_live_iterator_peekable_next_if_report(raw: &str) -> String {
    format!("dead-live-iterator-peekable-next-if-report:{raw}")
}
