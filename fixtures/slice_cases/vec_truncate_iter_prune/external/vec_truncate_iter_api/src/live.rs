pub fn selected_vec_truncate_iter_report(raw: &str) -> String {
    vec_truncate_iter_model::selected_vec_truncate_iter(raw)
}

pub fn dead_live_vec_truncate_iter_report(raw: &str) -> String {
    format!("dead-live-vec-truncate-iter:{raw}")
}
