pub fn selected_index_report(raw: &str) -> String {
    index_model::selected_index(raw)
}

pub fn dead_live_index_report(raw: &str) -> String {
    format!("dead-live-index:{raw}")
}
