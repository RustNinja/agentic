pub fn selected_vec_leak_iter_report(raw: &str) -> String {
    vec_leak_iter_model::selected_vec_leak_iter(raw)
}

pub fn dead_live_vec_leak_iter_report(raw: &str) -> String {
    format!("dead-live-vec-leak-iter:{raw}")
}
