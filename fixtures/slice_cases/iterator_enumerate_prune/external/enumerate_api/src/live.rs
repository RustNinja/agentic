pub fn selected_enumerate_report(raw: &str) -> String {
    enumerate_model::selected_enumerate(raw)
}

pub fn dead_live_enumerate_report(raw: &str) -> String {
    format!("dead-live-enumerate:{raw}")
}
