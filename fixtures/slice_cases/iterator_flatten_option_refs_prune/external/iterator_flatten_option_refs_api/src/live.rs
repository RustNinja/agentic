pub fn selected_iterator_flatten_option_refs_report(raw: &str) -> String {
    iterator_flatten_option_refs_model::selected_iterator_flatten_option_refs(raw)
}

pub fn dead_live_iterator_flatten_option_refs_report(raw: &str) -> String {
    format!("dead-iterator-flatten-option-refs-live-report:{raw}")
}
