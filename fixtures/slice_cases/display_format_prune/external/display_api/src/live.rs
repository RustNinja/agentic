pub fn selected_display_report(raw: &str) -> String {
    display_model::selected_display(raw)
}

pub fn dead_live_display_report(raw: &str) -> String {
    format!("dead-live-display:{raw}")
}
