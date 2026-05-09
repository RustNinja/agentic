pub fn selected_vec_retain_mut_report(raw: &str) -> String {
    vec_retain_mut_model::selected_vec_retain_mut(raw)
}

pub fn dead_live_vec_retain_mut_report(raw: &str) -> String {
    format!("dead-vec-retain-mut-live-report:{raw}")
}
