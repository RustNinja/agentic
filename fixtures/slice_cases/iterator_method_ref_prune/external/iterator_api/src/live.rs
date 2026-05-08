pub fn selected_iterator_report(raw: &str) -> String {
    iterator_model::selected_iterator(raw)
}

pub fn dead_live_iterator_report(raw: &str) -> String {
    format!("dead-live-iterator:{raw}")
}
