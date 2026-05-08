pub fn selected_filter_report(raw: &str) -> String {
    filter_model::selected_filter(raw)
}

pub fn dead_live_filter_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
