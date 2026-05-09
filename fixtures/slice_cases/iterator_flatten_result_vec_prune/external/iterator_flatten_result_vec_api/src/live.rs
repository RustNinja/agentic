pub fn selected_iterator_flatten_result_vec_report(raw: &str) -> String {
    iterator_flatten_result_vec_model::selected_iterator_flatten_result_vec(raw)
}

pub fn dead_live_iterator_flatten_result_vec_report(raw: &str) -> String {
    format!("dead-iterator-flatten-result-vec-live-report:{raw}")
}
