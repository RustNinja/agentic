pub fn selected_vec_get_report(raw: &str) -> String {
    vec_get_model::selected_vec_get(raw)
}

pub fn dead_live_vec_get_report(raw: &str) -> String {
    format!("dead-vec-get-live-report:{raw}")
}
