pub fn selected_vec_reverse_iter_report(raw: &str) -> String {
    vec_reverse_iter_model::selected_vec_reverse_iter(raw)
}

pub fn dead_live_vec_reverse_iter_report(raw: &str) -> String {
    format!("dead-live-vec-reverse-iter:{raw}")
}
