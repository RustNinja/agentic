pub fn selected_vec_fill_with_iter_report(raw: &str) -> String {
    vec_fill_with_iter_model::selected_vec_fill_with_iter(raw)
}

pub fn dead_live_vec_fill_with_iter_report(raw: &str) -> String {
    format!("dead-live-vec-fill-with-iter:{raw}")
}
