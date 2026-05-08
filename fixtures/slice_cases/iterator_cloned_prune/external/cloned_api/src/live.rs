pub fn selected_cloned_report(raw: &str) -> String {
    cloned_model::selected_cloned(raw)
}

pub fn dead_live_cloned_report(raw: &str) -> String {
    format!("dead-live-cloned:{raw}")
}
