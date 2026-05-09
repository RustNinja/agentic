pub fn selected_vec_into_iter_report(raw: &str) -> String {
    vec_into_iter_model::selected_vec_into_iter(raw)
}

pub fn dead_live_vec_into_iter_report(raw: &str) -> String {
    format!("dead-vec-into-iter-live-report:{raw}")
}
