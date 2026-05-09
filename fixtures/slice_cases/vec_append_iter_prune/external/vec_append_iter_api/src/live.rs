pub fn selected_vec_append_iter_report(raw: &str) -> String {
    vec_append_iter_model::selected_vec_append_iter(raw)
}

pub fn dead_live_vec_append_iter_report(raw: &str) -> String {
    format!("dead-live-vec-append-iter:{raw}")
}
