pub fn selected_vec_remove_report(raw: &str) -> String {
    vec_remove_model::selected_vec_remove(raw)
}

pub fn dead_live_vec_remove_report(raw: &str) -> String {
    format!("dead-vec-remove-live-report:{raw}")
}
