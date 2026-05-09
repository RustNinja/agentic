pub fn selected_vec_last_report(raw: &str) -> String {
    vec_last_model::selected_vec_last(raw)
}

pub fn dead_live_vec_last_report(raw: &str) -> String {
    format!("dead-vec-last-live-report:{raw}")
}
