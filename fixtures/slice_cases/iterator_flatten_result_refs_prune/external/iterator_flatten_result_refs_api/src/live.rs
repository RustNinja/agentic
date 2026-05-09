pub fn selected_iterator_flatten_result_refs_report(raw: &str) -> String {
    iterator_flatten_result_refs_model::selected_iterator_flatten_result_refs(raw)
}

pub fn dead_live_iterator_flatten_result_refs_report(raw: &str) -> String {
    format!("dead-iterator-flatten-result-refs-live-report:{raw}")
}
