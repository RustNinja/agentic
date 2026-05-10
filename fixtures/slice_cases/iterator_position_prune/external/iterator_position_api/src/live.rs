pub fn selected_iterator_position_report(raw: &str) -> String {
    iterator_position_model::selected_iterator_position(raw)
}

pub fn dead_live_iterator_position_report(raw: &str) -> String {
    format!("dead-live-iterator-position-report:{raw}")
}
