pub fn selected_settings_report(raw: &str) -> String {
    settings_model::selected_settings(raw)
}

pub fn dead_live_settings_report(raw: &str) -> String {
    format!("dead-live-settings:{raw}")
}
