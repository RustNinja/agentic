pub fn selected_vec_pop_report(raw: &str) -> String {
    vec_pop_model::selected_vec_pop(raw)
}

pub fn dead_live_vec_pop_report(raw: &str) -> String {
    format!("dead-vec-pop-live-report:{raw}")
}
