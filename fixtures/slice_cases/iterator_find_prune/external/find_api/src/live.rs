pub fn selected_find_report(raw: &str) -> String {
    find_model::selected_find(raw)
}

pub fn dead_live_find_report(raw: &str) -> String {
    format!("dead-find-live-report:{raw}")
}
