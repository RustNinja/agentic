pub fn selected_iterator_flatten_vec_vec_report(raw: &str) -> String {
    iterator_flatten_vec_vec_model::selected_iterator_flatten_vec_vec(raw)
}

pub fn dead_live_iterator_flatten_vec_vec_report(raw: &str) -> String {
    format!("dead-iterator-flatten-vec-vec-live-report:{raw}")
}
