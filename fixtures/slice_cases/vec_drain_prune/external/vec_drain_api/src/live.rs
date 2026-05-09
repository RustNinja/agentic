pub fn selected_vec_drain_report(raw: &str) -> String {
    vec_drain_model::selected_vec_drain(raw)
}

pub fn dead_live_vec_drain_report(raw: &str) -> String {
    format!("dead-vec-drain-live-report:{raw}")
}
