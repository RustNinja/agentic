pub fn selected_projection_report(raw: &str) -> String {
    projection_model::selected_projection(raw)
}

pub fn dead_live_projection_report(raw: &str) -> String {
    format!("dead-live-projection:{raw}")
}
