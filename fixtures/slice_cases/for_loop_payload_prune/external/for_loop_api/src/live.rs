pub fn selected_for_loop_report(raw: &str) -> String {
    for_loop_model::selected_for_loop(raw)
}

pub fn dead_live_for_loop_report(raw: &str) -> String {
    format!("dead-live-for-loop:{raw}")
}
