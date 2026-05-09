pub fn selected_vec_iter_find_report(raw: &str) -> String {
    vec_iter_find_model::selected_vec_iter_find(raw)
}

pub fn dead_live_vec_iter_find_report(raw: &str) -> String {
    format!("dead-vec-iter-find-live-report:{raw}")
}
