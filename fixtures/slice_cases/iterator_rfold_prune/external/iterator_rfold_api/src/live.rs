pub fn selected_iterator_rfold_report(raw: &str) -> String {
    iterator_rfold_model::selected_iterator_rfold(raw)
}

pub fn dead_live_iterator_rfold_report(raw: &str) -> String {
    format!("dead-live-iterator-rfold-report:{raw}")
}
