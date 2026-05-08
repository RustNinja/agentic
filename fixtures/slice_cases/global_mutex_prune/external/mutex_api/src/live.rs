pub fn selected_mutex_report(raw: &str) -> String {
    mutex_model::selected_mutex(raw)
}

pub fn dead_live_mutex_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
