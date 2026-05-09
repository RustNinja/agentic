pub fn selected_vec_first_report(raw: &str) -> String {
    vec_first_model::selected_vec_first(raw)
}

pub fn dead_live_vec_first_report(raw: &str) -> String {
    format!("dead-vec-first-live-report:{raw}")
}
