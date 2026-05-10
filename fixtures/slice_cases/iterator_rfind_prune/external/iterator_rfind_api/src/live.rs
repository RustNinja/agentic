pub fn selected_iterator_rfind_report(raw: &str) -> String {
    iterator_rfind_model::selected_iterator_rfind(raw)
}

pub fn dead_live_iterator_rfind_report(raw: &str) -> String {
    format!("dead-live-iterator-rfind-report:{raw}")
}
